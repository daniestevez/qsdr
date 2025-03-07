use super::chan::{
    self, AVAILABLE_SHIFT, Common, MAX_PENDING_SLOTS, RECEIVER_SLEEPING, TRANSMITTER_DROPPED, Waker,
};
use libc::{FUTEX_PRIVATE_FLAG, FUTEX_WAIT, FUTEX_WAKE, SYS_futex, syscall};
use std::sync::atomic::{
    Ordering::{AcqRel, Acquire, Relaxed},
    fence,
};

#[derive(Debug)]
pub struct Sender<T>(chan::Sender<T, FutexWaker>);

impl<T> Sender<T> {
    pub fn send(&mut self, value: T) {
        self.0.send(value)
    }
}

#[derive(Debug)]
pub struct Receiver<T>(chan::Receiver<T, FutexWaker>);

unsafe impl<T: Send> Send for Receiver<T> {}
// Receiver can be Sync, because the receive methods use &mut self,
// so it is not possible to have multiple consumers by sharing references
unsafe impl<T: Send> Sync for Receiver<T> {}

#[derive(Debug, Clone, Default)]
struct FutexWaker {}

impl Waker for FutexWaker {
    fn wake<T>(common: &Common<T, FutexWaker>) {
        unsafe {
            syscall(
                SYS_futex,
                common.shared().as_ptr().cast_const(),
                FUTEX_WAKE | FUTEX_PRIVATE_FLAG,
                1,
            )
        };
    }
}

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = chan::channel(size);
    (Sender(tx), Receiver(rx))
}

const RECEIVER_SPINS: usize = 1 << 13;

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        self.0.recv_futex_waker()
    }
}

impl<T> chan::Receiver<T, FutexWaker> {
    fn recv_futex_waker(&mut self) -> Option<T> {
        let mut available = self.available;
        if available >> AVAILABLE_SHIFT == 0 {
            for _ in 0..RECEIVER_SPINS {
                available = self.common.shared().load(Relaxed);
                if available >> AVAILABLE_SHIFT != self.clear_pending {
                    break;
                }
                std::hint::spin_loop();
            }
            if available >> AVAILABLE_SHIFT == self.clear_pending {
                loop {
                    available = self.common.shared().load(Relaxed);
                    if available >> AVAILABLE_SHIFT != self.clear_pending {
                        break;
                    }
                    if available & TRANSMITTER_DROPPED != 0 {
                        // channel is empty and sender was dropped
                        return None;
                    }
                    self.common.shared().fetch_or(RECEIVER_SLEEPING, Relaxed);
                    unsafe {
                        syscall(
                            SYS_futex,
                            self.common.shared().as_ptr().cast_const(),
                            FUTEX_WAIT | FUTEX_PRIVATE_FLAG,
                            available | RECEIVER_SLEEPING,
                            std::ptr::null::<libc::timespec>(),
                        )
                    };
                }
            }
            self.available = available - (self.clear_pending << AVAILABLE_SHIFT);
            if size_of::<T>() != 0 {
                // if T is ZST the read below is a noop, so it doesn't
                // need to be synchronized
                fence(Acquire);
            }
        }
        let value = unsafe {
            self.common
                .item_buf()
                .add((self.read_idx & self.common.mask) as usize)
                .read()
        };
        self.clear_pending += 1;
        if self.clear_pending == MAX_PENDING_SLOTS {
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
                .common
                .shared()
                .fetch_sub(MAX_PENDING_SLOTS << AVAILABLE_SHIFT, ordering);
            self.available = old_shared - (MAX_PENDING_SLOTS << AVAILABLE_SHIFT);
            self.clear_pending = 0;
        } else {
            self.available -= 1 << AVAILABLE_SHIFT;
        }
        self.read_idx = self.read_idx.wrapping_add(1);
        Some(value)
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
        let received = std::iter::repeat_with(|| rx.recv().unwrap())
            .take(cap)
            .collect::<Vec<_>>();
        let expected = (0..cap).collect::<Vec<_>>();
        assert_eq!(received, expected);
        sender_thread.join().unwrap();
        assert!(rx.recv().is_none());
    }
}
