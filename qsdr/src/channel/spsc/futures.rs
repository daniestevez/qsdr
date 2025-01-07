use super::chan::{
    self, Common, Waker, AVAILABLE_SHIFT, MAX_PENDING_SLOTS, RECEIVER_SLEEPING, TRANSMITTER_DROPPED,
};
use futures::stream::Stream;
use std::{
    future::poll_fn,
    pin::Pin,
    sync::atomic::{
        fence,
        Ordering::{AcqRel, Acquire, Relaxed},
    },
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Sender<T>(chan::Sender<T, AtomicWaker>);

impl<T> Sender<T> {
    pub fn send(&mut self, value: T) {
        self.0.send(value)
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
            // Normally this loop only spins once, but it might spin multiple times
            // if the waker is waked before this returns Poll::Pending (in which
            // case the wake might have been lost).
            available = self.0.common.shared().load(Relaxed);
            loop {
                if available >> AVAILABLE_SHIFT != self.0.clear_pending {
                    break;
                }
                if available & TRANSMITTER_DROPPED != 0 {
                    // channel is empty and sender was dropped
                    return Poll::Ready(None);
                }
                self.0.common.shared().fetch_or(RECEIVER_SLEEPING, Relaxed);
                self.0.common.waker().0.register(cx.waker());
                // Check that the shared atomic is still what we expect. This
                // avoids lost wakes due to the following race condition: after
                // we have checked that there are no available messages, the
                // sender might have added a new message and seen that the
                // receiver was not sleeping, so it did not send a wake.
                let old_available = available;
                available = self.0.common.shared().load(Relaxed);
                if available == old_available | RECEIVER_SLEEPING {
                    return Poll::Pending;
                }
            }
            self.0.available = available - (self.0.clear_pending << AVAILABLE_SHIFT);
            if size_of::<T>() != 0 {
                // if T is ZST the read below is a noop, so it doesn't
                // need to be synchronized
                fence(Acquire);
            }
        }
        let value = unsafe {
            self.0
                .common
                .item_buf()
                .add((self.0.read_idx & self.0.common.mask) as usize)
                .read()
        };
        self.0.clear_pending += 1;
        if self.0.clear_pending == MAX_PENDING_SLOTS {
            let ordering = if size_of::<T>() == 0 {
                // if T is ZST, reads and writes are a noop, so synchronization
                // is not needed
                Relaxed
            } else {
                // Acquire because writes into the buffer by the sender need to
                // happen before reads by future recv() calls. Release because the
                // read of this item from the buffer needs to happen before an
                // overwrite of the same slot by the sender.
                AcqRel
            };
            let old_shared = self
                .0
                .common
                .shared()
                .fetch_sub(MAX_PENDING_SLOTS << AVAILABLE_SHIFT, ordering);
            self.0.available = old_shared - (MAX_PENDING_SLOTS << AVAILABLE_SHIFT);
            self.0.clear_pending = 0;
        } else {
            self.0.available -= 1 << AVAILABLE_SHIFT;
        }
        self.0.read_idx = self.0.read_idx.wrapping_add(1);
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
    fn channel() {
        let cap = 4096;
        let (mut tx, mut rx) = super::channel(cap);
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
}
