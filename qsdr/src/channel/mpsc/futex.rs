use super::chan::{
    self, Common, Waker, AVAILABLE_SHIFT, MAX_PENDING_SLOTS, RECEIVER_SLEEPING,
    TRANSMITTERS_DROPPED, WRITE_IDX_SHIFT,
};
use libc::{syscall, SYS_futex, FUTEX_PRIVATE_FLAG, FUTEX_WAIT, FUTEX_WAKE};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

#[derive(Debug)]
pub struct Sender<T>(chan::Sender<T, FutexWaker>);

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
                available = self.common.shared().load(Relaxed) & ((1 << WRITE_IDX_SHIFT) - 1);
                if available >> AVAILABLE_SHIFT != self.clear_pending {
                    break;
                }
                std::hint::spin_loop();
            }
            if available >> AVAILABLE_SHIFT == self.clear_pending {
                loop {
                    let shared = self.common.shared().load(Relaxed);
                    available = shared & ((1 << WRITE_IDX_SHIFT) - 1);
                    if available >> AVAILABLE_SHIFT != self.clear_pending {
                        break;
                    }
                    if shared & TRANSMITTERS_DROPPED != 0 {
                        // channel is empty and all senders were dropped
                        return None;
                    }
                    self.common.shared().fetch_or(RECEIVER_SLEEPING, Relaxed);
                    unsafe {
                        syscall(
                            SYS_futex,
                            self.common.shared().as_ptr().cast_const(),
                            FUTEX_WAIT | FUTEX_PRIVATE_FLAG,
                            shared | RECEIVER_SLEEPING,
                            std::ptr::null::<libc::timespec>(),
                        )
                    };
                }
            }
            self.available = available - (self.clear_pending << AVAILABLE_SHIFT);
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
                .common
                .slot_buf()
                .add((self.read_idx & self.common.mask) as usize)
                .as_ptr();
            // spin until the sequence on the item to read matches the expected value
            while (*slot).sequence.load(ordering) != self.read_idx {}
            std::ptr::read(&raw const (*slot).value)
        };
        self.clear_pending += 1;
        if self.clear_pending == MAX_PENDING_SLOTS {
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
                .common
                .shared()
                .fetch_sub(MAX_PENDING_SLOTS << AVAILABLE_SHIFT, ordering);
            self.available = (old_shared - (MAX_PENDING_SLOTS << AVAILABLE_SHIFT))
                & ((1 << WRITE_IDX_SHIFT) - 1);
            self.clear_pending = 0;
        } else {
            self.available -= 1 << AVAILABLE_SHIFT;
        }
        self.read_idx = (self.read_idx.wrapping_add(1) << WRITE_IDX_SHIFT) >> WRITE_IDX_SHIFT;
        Some(value)
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
        let received = std::iter::repeat_with(|| rx.recv().unwrap())
            .take(cap)
            .collect::<Vec<_>>();
        let expected = (0..cap).collect::<Vec<_>>();
        assert_eq!(received, expected);
        sender_thread.join().unwrap();
        assert!(rx.recv().is_none());
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
        let mut received = std::iter::repeat_with(|| rx.recv().unwrap())
            .take(cap)
            .collect::<Vec<_>>();
        received.sort_unstable();
        let expected = (0..cap).collect::<Vec<_>>();
        assert_eq!(received, expected);
        for thread in sender_threads.into_iter() {
            thread.join().unwrap();
        }
        assert!(rx.recv().is_none());
    }
}
