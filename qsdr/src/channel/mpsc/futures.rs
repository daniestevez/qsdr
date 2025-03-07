use super::chan::{
    self, AVAILABLE_SHIFT, Common, MAX_PENDING_SLOTS, RECEIVER_SLEEPING, TRANSMITTERS_DROPPED,
    WRITE_IDX_SHIFT, Waker,
};
use futures::stream::Stream;
use std::{
    future::poll_fn,
    pin::Pin,
    sync::atomic::Ordering::{Acquire, Relaxed, Release},
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Sender<T>(chan::Sender<T, AtomicWaker>);

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        self.0.send(value)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Sender(self.0.clone())
    }
}

#[derive(Debug)]
pub struct Receiver<T>(chan::Receiver<T, AtomicWaker>);

unsafe impl<T: Send> Send for Receiver<T> {}
// Receiver can be Sync, because the receive methods use &mut self,
// so it is not possible to have multiple consumers by sharing references
unsafe impl<T: Send> Sync for Receiver<T> {}

#[derive(Debug, Default)]
struct AtomicWaker(futures::task::AtomicWaker);

impl Waker for AtomicWaker {
    fn wake<T>(common: &Common<T, AtomicWaker>) {
        common.waker().0.wake();
    }
}

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = chan::channel(size);
    (Sender(tx), Receiver(rx))
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        poll_fn(|cx| self.poll(cx)).await
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let mut available = self.0.available;
        if available >> AVAILABLE_SHIFT == 0 {
            // Normally this loop spins at most once, but it might spin multiple
            // times if the waker is waked before this returns Poll::Pending (in
            // which case the wake might have been lost).
            let mut shared = self.0.common.shared().load(Relaxed);
            loop {
                available = shared & ((1 << WRITE_IDX_SHIFT) - 1);
                if available >> AVAILABLE_SHIFT != self.0.clear_pending {
                    break;
                }
                if shared & TRANSMITTERS_DROPPED != 0 {
                    // channel is empty and all senders were dropped
                    return Poll::Ready(None);
                }
                self.0.common.shared().fetch_or(RECEIVER_SLEEPING, Relaxed);
                self.0.common.waker().0.register(cx.waker());
                // Check that the shared atomic is still what we expect. This
                // avoids lost wakes due to the following race condition: after
                // we have checked that there are no available messages, the
                // sender might have added a new message and seen that the
                // receiver was not sleeping, so it did not send a wake.
                let old_shared = shared;
                shared = self.0.common.shared().load(Relaxed);
                if shared == old_shared | RECEIVER_SLEEPING {
                    return Poll::Pending;
                }
            }
            self.0.available = available - (self.0.clear_pending << AVAILABLE_SHIFT);
        }
        let ordering = if size_of::<T>() == 0 {
            // if T is ZST, the read below is a noop, so it doesn't need to be
            // synchronized
            Relaxed
        } else {
            // Acquire because the write of the value in the slot by the sender
            // needs to happen before the read
            Acquire
        };
        let value = unsafe {
            let slot = self
                .0
                .common
                .slot_buf()
                .add((self.0.read_idx & self.0.common.mask) as usize)
                .as_ptr();
            // spin until the sequence on the item to read matches the expected value
            while (*slot).sequence.load(ordering) != self.0.read_idx {}
            std::ptr::read(&raw const (*slot).value)
        };
        self.0.clear_pending += 1;
        if self.0.clear_pending == MAX_PENDING_SLOTS {
            let ordering = if size_of::<T>() == 0 {
                // if T is ZST, reads and writes are a noop, so synchronization
                // is not needed
                Relaxed
            } else {
                // Release because the read of this item from the buffer needs
                // to happen before an overwrite of the same slot by the sender.
                Release
            };
            let old_shared = self
                .0
                .common
                .shared()
                .fetch_sub(MAX_PENDING_SLOTS << AVAILABLE_SHIFT, ordering);
            self.0.available = (old_shared - (MAX_PENDING_SLOTS << AVAILABLE_SHIFT))
                & ((1 << WRITE_IDX_SHIFT) - 1);
            self.0.clear_pending = 0;
        } else {
            self.0.available -= 1 << AVAILABLE_SHIFT;
        }
        self.0.read_idx = (self.0.read_idx.wrapping_add(1) << WRITE_IDX_SHIFT) >> WRITE_IDX_SHIFT;
        Poll::Ready(Some(value))
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.poll(cx)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn single_sender() {
        let cap = 4096;
        let (tx, mut rx) = super::channel(cap);
        let sender_thread = std::thread::spawn(move || {
            for n in 0..cap {
                tx.send(n);
            }
        });
        futures::executor::block_on(async move {
            let mut received = Vec::with_capacity(cap);
            for _ in 0..cap {
                received.push(rx.recv().await.unwrap());
            }
            let expected = (0..cap).collect::<Vec<_>>();
            assert_eq!(received, expected);
            sender_thread.join().unwrap();
            assert!(rx.recv().await.is_none());
        });
    }

    #[test]
    fn multiple_senders() {
        let cap = 4096;
        let senders = 8;
        let (tx, mut rx) = super::channel(cap);
        let sender_threads = (0..senders)
            .map(|thread_num| {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    for n in (0..cap).skip(thread_num).step_by(senders) {
                        tx.send(n);
                    }
                })
            })
            .collect::<Vec<_>>();
        // this drop is needed somewhere before the end of the test, or
        // otherwise the receiver will hang waiting for more data
        drop(tx);
        futures::executor::block_on(async move {
            let mut received = Vec::with_capacity(cap);
            for _ in 0..cap {
                received.push(rx.recv().await.unwrap());
            }
            received.sort_unstable();
            let expected = (0..cap).collect::<Vec<_>>();
            assert_eq!(received, expected);
            for thread in sender_threads.into_iter() {
                thread.join().unwrap();
            }
            assert!(rx.recv().await.is_none());
        });
    }
}
