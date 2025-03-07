// Part of this code is taken from futures-rs/futures-executor

use futures::{executor::enter, pin_mut};
use futures_task::{ArcWake, waker_ref};
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    thread::{self, Thread},
};

struct ThreadNotify {
    /// The (single) executor thread.
    thread: Thread,
    /// A flag to ensure a wakeup (i.e. `unpark()`) is not "forgotten"
    /// before the next `park()`, which may otherwise happen if the code
    /// being executed as part of the future(s) being polled makes use of
    /// park / unpark calls of its own, i.e. we cannot assume that no other
    /// code uses park / unpark on the executing `thread`.
    unparked: AtomicBool,
}

std::thread_local! {
    static CURRENT_THREAD_NOTIFY: Arc<ThreadNotify> = Arc::new(ThreadNotify {
        thread: thread::current(),
        unparked: AtomicBool::new(false),
    });
}

impl ArcWake for ThreadNotify {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Make sure the wakeup is remembered until the next `park()`.
        let unparked = arc_self.unparked.swap(true, Ordering::Release);
        if !unparked {
            // If the thread has not been unparked yet, it must be done
            // now. If it was actually parked, it will run again,
            // otherwise the token made available by `unpark`
            // may be consumed before reaching `park()`, but `unparked`
            // ensures it is not forgotten.
            arc_self.thread.unpark();
        }
    }
}

// Set up and run a basic single-threaded spawner loop, invoking `f` on each
// turn. `f` is invoked multiple times before parking if it returns
// Poll::Pending, with the hope that it becomes ready after some busy waiting
// and we don't need to sleep the thread.
fn run_executor<T, F: FnMut(&mut Context<'_>) -> Poll<T>>(mut f: F) -> T {
    let _enter = enter().expect("cannot execute executor from within another executor");

    CURRENT_THREAD_NOTIFY.with(|thread_notify| {
        let waker = waker_ref(thread_notify);
        let mut cx = Context::from_waker(&waker);
        loop {
            const SPINS: usize = 1 << 10;
            for _ in 0..SPINS {
                if let Poll::Ready(t) = f(&mut cx) {
                    return t;
                }
            }

            // Wait for a wakeup.
            while !thread_notify.unparked.swap(false, Ordering::Acquire) {
                // No wakeup occurred. It may occur now, right before parking,
                // but in that case the token made available by `unpark()`
                // is guaranteed to still be available and `park()` is a no-op.
                thread::park();
            }
        }
    })
}

/// Run a future to completion on the current thread.
///
/// This function will block the caller until the given future has completed.
pub fn block_on<F: Future>(f: F) -> F::Output {
    pin_mut!(f);
    run_executor(|cx| f.as_mut().poll(cx))
}
