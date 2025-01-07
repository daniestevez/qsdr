use super::{Channel, RefReceiver};
use crate::channel::{mpsc::futures as mpsc, spsc::futures as spsc};
use anyhow::Result;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    borrow::Borrow,
    mem::size_of,
    ptr::NonNull,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering::{Acquire, Relaxed, Release},
        },
        Arc,
    },
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct SpBroadcast {}

impl Channel for SpBroadcast {
    type Sender<T> = Sender<T>;

    type Receiver<T> = Receiver<T>;

    type ReturnReceiver<T> = mpsc::Receiver<T>;

    type SenderSeed<T> = SenderSeed<T>;

    type ReceiverSeed<T> = ReceiverSeed<T>;

    type ReturnReceiverSeed<T> = super::mpsc::ReceiverSeed<T>;

    fn connect<T, I: Iterator<Item = T>>(
        size: usize,
        source: &mut Self::SenderSeed<T>,
        dest: &mut Self::ReceiverSeed<T>,
        return_dest: &mut Self::ReturnReceiverSeed<T>,
        inject_messages: I,
    ) -> Result<()> {
        anyhow::ensure!(dest.0.is_none(), "destination is already connected");
        let return_tx = if let Some(return_seed) = &return_dest.0 {
            // return channel already exists
            return_seed.sender.clone()
        } else {
            // return channel does not exist yet
            let (return_tx, return_rx) = mpsc::channel(size);
            for message in inject_messages.take(size) {
                return_tx.send(message);
            }
            return_dest.0.replace(super::mpsc::RxSeed {
                receiver: return_rx,
                sender: return_tx.clone(),
            });
            return_tx
        };
        let (forward_tx, forward_rx) = spsc::channel(size);
        if source.senders.is_empty() {
            // allocate buffer
            let buffer_size = size.next_power_of_two();
            let layout = Layout::array::<Slot<T>>(buffer_size).unwrap();
            let ptr = unsafe { alloc(layout) };
            let Some(ptr) = NonNull::new(ptr.cast::<Slot<T>>()) else {
                handle_alloc_error(layout);
            };
            for n in 0..buffer_size {
                // SAFETY: the buffer has been allocated immediately above
                unsafe {
                    std::ptr::write(
                        &raw mut (*ptr.add(n).as_ptr()).refcount,
                        AtomicUsize::new(usize::MAX),
                    );
                }
            }
            let buffer = Buffer {
                buffer: ptr,
                mask: buffer_size - 1,
            };
            let previous = source.buffer.replace(Arc::new(buffer));
            assert!(previous.is_none());
        }
        source.senders.push(forward_tx);
        dest.0.replace(Receiver {
            receiver: forward_rx,
            return_sender: return_tx,
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    senders: Vec<spsc::Sender<Message<T>>>,
    buffer: Arc<Buffer<T>>,
    buffer_idx: usize,
}

#[derive(Debug)]
struct Buffer<T> {
    buffer: NonNull<Slot<T>>,
    mask: usize,
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        let size = self.mask + 1;
        if std::mem::needs_drop::<T>() {
            for n in 0..size {
                // SAFETY: the offset of 'n' is in-bounds of the allocation, and
                // if refcount is not usize::MAX, then slot.value contains a 'T'
                // which can be dropped.
                unsafe {
                    let slot = self.buffer.add(n).as_ptr();
                    // Relaxed ordering can be used here. The value that is
                    // dropped in place below was either written by Sender::send
                    // on the same thread (if the Sender is the last object to
                    // drop the Buffer) or already synchronized with this thread
                    // by means of the channel (if a RefEnvelope is the last
                    // object to drop the Buffer).
                    let refcount = (*slot).refcount.load(Relaxed);
                    assert_ne!(refcount, 0);
                    if refcount != usize::MAX {
                        std::ptr::drop_in_place(&raw mut (*slot).value);
                    }
                }
            }
        }
        let layout = Layout::array::<Slot<T>>(size).unwrap();
        // SAFETY: the allocation was constructed with the same layout and will
        // not be used any more.
        unsafe {
            dealloc(self.buffer.as_ptr().cast::<u8>(), layout);
        }
    }
}

#[derive(Debug)]
struct Slot<T> {
    value: T,
    // refcount is usize::MAX if the slot is vacant
    refcount: AtomicUsize,
}

#[derive(Debug)]
struct Message<T> {
    slot: NonNull<Slot<T>>,
    buffer: Arc<Buffer<T>>,
}

impl<T> super::Sender<T> for Sender<T> {
    fn send(&mut self, value: T) {
        // SAFETY: the calculated offset is in-bounds of the allocation. If
        // refcount contains usize::MAX, the slot is vacant and can be
        // overwritten.
        unsafe {
            let slot = self.buffer.buffer.add(self.buffer_idx & self.buffer.mask);
            self.buffer_idx += 1;
            let ptr = slot.as_ptr();
            let ordering = if size_of::<T>() == 0 {
                // if T is ZST, then relaxed ordering can be used, since the ptr
                // write is a noop.
                Relaxed
            } else {
                // acquire ordering to synchronize with the write below
                Acquire
            };
            if (*ptr).refcount.load(ordering) != usize::MAX {
                panic!("tried to store value in a non-vacant slot");
            }
            std::ptr::write(&raw mut (*ptr).value, value);
            // relaxed ordering can be used here, since sending a message
            // through the channel already has release-acquire semantics
            (*ptr).refcount.store(self.senders.len(), Relaxed);
            for sender in &mut self.senders {
                sender.send(Message {
                    slot,
                    buffer: Arc::clone(&self.buffer),
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    receiver: spsc::Receiver<Message<T>>,
    return_sender: mpsc::Sender<T>,
}

impl<T> RefReceiver<T> for Receiver<T> {
    type Ref<'a>
        = RefEnvelope<'a, T>
    where
        T: 'a;

    async fn ref_recv(&mut self) -> Option<Self::Ref<'_>> {
        let message = self.receiver.recv().await?;
        Some(RefEnvelope {
            slot: message.slot,
            _buffer: message.buffer,
            return_sender: &self.return_sender,
        })
    }
}

#[derive(Debug)]
pub struct RefEnvelope<'a, T> {
    slot: NonNull<Slot<T>>,
    // _buffer is only used to keep alive the buffer allocation at least until
    // RefEnvelope is dropped
    _buffer: Arc<Buffer<T>>,
    return_sender: &'a mpsc::Sender<T>,
}

impl<T> Borrow<T> for RefEnvelope<'_, T> {
    fn borrow(&self) -> &T {
        // SAFETY: while RefEnvelope is alive, slot points to a valid T
        unsafe { &self.slot.as_ref().value }
    }
}

impl<T> AsRef<T> for RefEnvelope<'_, T> {
    fn as_ref(&self) -> &T {
        self.borrow()
    }
}

impl<T> Drop for RefEnvelope<'_, T> {
    fn drop(&mut self) {
        let ptr = self.slot.as_ptr();
        // SAFETY: if refcount is 1, then the slot is no longer in use, so it
        // can be read and sent to the return_sender.
        unsafe {
            if (*ptr).refcount.fetch_sub(1, Relaxed) == 1 {
                // this was the last RefEnvelope holding a reference to the object
                let value = std::ptr::read(&raw mut (*ptr).value);
                // mark slot as vacant
                let ordering = if size_of::<T>() == 0 {
                    Relaxed
                } else {
                    // synchronize the reading of value with the writing on a
                    // vacant slot on Sender::send
                    Release
                };
                (*ptr).refcount.store(usize::MAX, ordering);
                self.return_sender.send(value);
            }
        }
    }
}

#[derive(Debug)]
pub struct SenderSeed<T> {
    senders: Vec<spsc::Sender<Message<T>>>,
    buffer: Option<Arc<Buffer<T>>>,
    buffer_idx: usize,
}

impl<T> Default for SenderSeed<T> {
    fn default() -> Self {
        Self {
            senders: Vec::new(),
            buffer: None,
            buffer_idx: 0,
        }
    }
}

impl<T> TryFrom<SenderSeed<T>> for Sender<T> {
    type Error = anyhow::Error;

    fn try_from(value: SenderSeed<T>) -> Result<Sender<T>> {
        anyhow::ensure!(!value.senders.is_empty(), "port is not connected");
        Ok(Sender {
            senders: value.senders,
            buffer: value.buffer.unwrap(),
            buffer_idx: value.buffer_idx,
        })
    }
}

#[derive(Debug)]
pub struct ReceiverSeed<T>(Option<Receiver<T>>);

impl<T> Default for ReceiverSeed<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> TryFrom<ReceiverSeed<T>> for Receiver<T> {
    type Error = anyhow::Error;

    fn try_from(value: ReceiverSeed<T>) -> Result<Receiver<T>> {
        value
            .0
            .ok_or_else(|| anyhow::anyhow!("port is not connected"))
    }
}
