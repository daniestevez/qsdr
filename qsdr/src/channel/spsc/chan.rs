use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{
        AtomicU32,
        Ordering::{AcqRel, Relaxed, Release},
    },
};

pub const RECEIVER_SLEEPING: u32 = 1 << 0;
pub const TRANSMITTER_DROPPED: u32 = 1 << 1;
pub const AVAILABLE_SHIFT: u32 = 2;

pub const MAX_PENDING_SLOTS: u32 = 128;

const ENDPOINT_DROPPED_OFFSET: isize = 1;
const READ_IDX_OFFSET: isize = 2;

// align shared Atomic32's to a cache line, and make them occupy a full cache line
const SHARED_ATOMICS_ALIGN: usize = 64;
const SHARED_ATOMICS_SIZE: usize = SHARED_ATOMICS_ALIGN;

#[derive(Debug)]
pub struct Sender<T, W: Waker> {
    pub common: Common<T, W>,
    write_idx: u32,
}

unsafe impl<T: Send, W: Waker> Send for Sender<T, W> {}
// Sender can be Sync, because the send method uses &mut self, so it is not
// possible to have multiple consumers by sharing references
unsafe impl<T: Send, W: Waker> Sync for Sender<T, W> {}

#[derive(Debug)]
pub struct Receiver<T, W> {
    pub common: Common<T, W>,
    pub read_idx: u32,
    pub available: u32,
    pub clear_pending: u32,
}

pub fn channel<T, W: Waker>(size: usize) -> (Sender<T, W>, Receiver<T, W>) {
    let common = Common::new(size);
    (
        Sender {
            common: common.clone(),
            write_idx: 0,
        },
        Receiver {
            common,
            read_idx: 0,
            available: 0,
            clear_pending: 0,
        },
    )
}

impl<T, W: Waker> Sender<T, W> {
    pub fn send(&mut self, value: T) {
        // these variables are used because otherwise the compiler loads the
        // values several times
        unsafe {
            self.common
                .item_buf()
                .add((self.write_idx & self.common.mask) as usize)
                .write(value)
        };
        let ordering = if size_of::<T>() == 0 {
            // if T is ZST, the write above has been a noop, so it doesn't need
            // to be synchronized
            Relaxed
        } else {
            // Acquire because the overwrite of buffer slots by future send()
            // calls needs to happen after the items in those slots have been
            // read by the receiver. Release because the write of this item in
            // the buffer needs to happen before it is read by the receiver.
            AcqRel
        };
        let old_shared = self
            .common
            .shared()
            .fetch_add(1 << AVAILABLE_SHIFT, ordering);
        assert!(
            old_shared >> AVAILABLE_SHIFT != self.common.mask,
            "send() called on a full channel"
        );
        if old_shared & RECEIVER_SLEEPING != 0 {
            self.common.shared().fetch_and(!RECEIVER_SLEEPING, Relaxed);
            W::wake(&self.common);
        }
        self.write_idx = self.write_idx.wrapping_add(1);
    }
}

#[derive(Debug)]
pub struct Common<T, W> {
    shared: NonNull<AtomicU32>,
    pub mask: u32,
    pub write_idx: u32,
    _phantom: PhantomData<(*mut T, *mut W)>,
}

pub trait Waker: Sized + Send + Default {
    fn wake<T>(common: &Common<T, Self>);
}

impl<T, W> Clone for Common<T, W> {
    fn clone(&self) -> Common<T, W> {
        Common {
            shared: self.shared,
            mask: self.mask,
            write_idx: self.write_idx,
            _phantom: PhantomData,
        }
    }
}

impl<T, W: Default> Common<T, W> {
    fn new(size: usize) -> Common<T, W> {
        // The size is amended to size + MAX_PENDING_SLOTS, because at least
        // MAX_PENDING_SLOTS - 1 additional slots can be not claimed to be free
        // by the receiver, and one additional slot is needed for the assertion
        // that detects a buffer full condition in the sender without causing
        // undefined behaviour.
        let size = size
            .checked_add(usize::try_from(MAX_PENDING_SLOTS).unwrap())
            .unwrap()
            .next_power_of_two();
        assert!(size != 0); // this happens when next_power_of_two overflows in release mode
        assert!(size <= 1 << (32 - AVAILABLE_SHIFT));
        let shared = {
            let layout = Self::shared_layout(size);
            let ptr = unsafe { alloc(layout) };
            let Some(ptr) = NonNull::new(ptr.cast::<AtomicU32>()) else {
                handle_alloc_error(layout);
            };
            unsafe {
                ptr.write(AtomicU32::new(0));
                ptr.offset(ENDPOINT_DROPPED_OFFSET).write(AtomicU32::new(0));
                // READ_IDX_OFFSET does not really need to be accessed atomically,
                // because it is synchronized by ENDPOINT_DROPPED_OFFSET, but we
                // still use atomic accesses for homogeneity
                ptr.offset(READ_IDX_OFFSET).write(AtomicU32::new(0));
                ptr.byte_add(Self::WAKER_BYTE_OFFSET)
                    .cast::<W>()
                    .write(W::default());
            }
            ptr
        };
        let mask = u32::try_from(size - 1).unwrap();
        Common {
            shared,
            mask,
            write_idx: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T, W> Common<T, W> {
    const WAKER_BYTE_OFFSET: usize =
        SHARED_ATOMICS_SIZE.next_multiple_of(std::mem::align_of::<W>());
    const ITEM_BUF_BYTE_OFFSET: usize = (Self::WAKER_BYTE_OFFSET + std::mem::size_of::<W>())
        .next_multiple_of(std::mem::align_of::<T>());

    fn shared_layout(size: usize) -> Layout {
        let atomics_layout =
            Layout::from_size_align(SHARED_ATOMICS_SIZE, SHARED_ATOMICS_ALIGN).unwrap();
        let waker_layout = Layout::new::<W>();
        let (combined_layout, offset) = atomics_layout.extend(waker_layout).unwrap();
        assert_eq!(offset, Self::WAKER_BYTE_OFFSET);
        let item_buf_layout = Layout::array::<T>(size).unwrap();
        let (combined_layout, offset) = combined_layout.extend(item_buf_layout).unwrap();
        assert_eq!(offset, Self::ITEM_BUF_BYTE_OFFSET);
        combined_layout
    }

    unsafe fn deallocate(&self) {
        if std::mem::needs_drop::<T>() {
            // drop elements still in buffer
            let mut read_idx = self.read_idx().load(Relaxed);
            let available = self.shared().load(Relaxed) >> AVAILABLE_SHIFT;
            for _ in 0..available {
                unsafe {
                    self.item_buf()
                        .add((read_idx & self.mask) as usize)
                        .drop_in_place()
                };
                read_idx = read_idx.wrapping_add(1);
            }
        }
        unsafe {
            self.shared
                .byte_add(Self::WAKER_BYTE_OFFSET)
                .cast::<W>()
                .drop_in_place()
        };
        let size = usize::try_from(self.mask).unwrap() + 1;
        unsafe {
            dealloc(self.shared.as_ptr().cast::<u8>(), Self::shared_layout(size))
        };
    }

    pub fn shared(&self) -> &AtomicU32 {
        unsafe { self.shared.as_ref() }
    }

    pub fn endpoint_dropped(&self) -> &AtomicU32 {
        unsafe { self.shared.offset(ENDPOINT_DROPPED_OFFSET).as_ref() }
    }

    pub fn read_idx(&self) -> &AtomicU32 {
        unsafe { self.shared.offset(READ_IDX_OFFSET).as_ref() }
    }

    pub fn waker(&self) -> &W {
        unsafe {
            self.shared
                .byte_add(Self::WAKER_BYTE_OFFSET)
                .cast::<W>()
                .as_ref()
        }
    }

    pub fn item_buf(&self) -> NonNull<T> {
        unsafe { self.shared.byte_add(Self::ITEM_BUF_BYTE_OFFSET).cast::<T>() }
    }
}

impl<T, W: Waker> Drop for Sender<T, W> {
    fn drop(&mut self) {
        // wake the receiver if it is sleeping
        let old_shared = self.common.shared().fetch_or(TRANSMITTER_DROPPED, Relaxed);
        if old_shared & RECEIVER_SLEEPING != 0 {
            W::wake(&self.common);
        }
        // Acquire ordering used here to establish a happens-before relationship
        // between storing at read_idx() in Receiver's drop and loading it on
        // deallocate. Release ordering used here to establish a happens-before
        // relationship between fetch_add in Sender::send and loading the value
        // on deallocate when run by Receiver's drop.
        let endpoint_dropped = self.common.endpoint_dropped().swap(1, AcqRel);
        if endpoint_dropped == 0 {
            // the receiver is not dropped yet; nothing to do
            return;
        }
        // the receiver is already dropped, so we need to clean up
        unsafe { self.common.deallocate() };
    }
}

impl<T, W> Drop for Receiver<T, W> {
    fn drop(&mut self) {
        self.common.read_idx().store(self.read_idx, Relaxed);
        let ordering = if size_of::<T>() == 0 {
            // if T is ZST, reads and writes are a noop, so synchronization
            // is not needed
            Relaxed
        } else {
            // Release because the read of these items from the buffer needs
            // to happen before an overwrite of the same slot by the sender
            // (the sender can still attempt to send even if the receiver is
            // dropped).
            Release
        };
        self.common
            .shared()
            .fetch_sub(self.clear_pending << AVAILABLE_SHIFT, ordering);
        // Release ordering used here to establish a happens-before relationship
        // between storing at read_idx() (and the fetch_sub at shared()), and
        // loading it on deallocate when run by Sender's drop. Acquire ordering
        // used here to establish a happens-before relationship between
        // fetch_add in Sender::send and loading the value on deallocate.
        let endpoint_dropped = self.common.endpoint_dropped().swap(1, AcqRel);
        if endpoint_dropped == 0 {
            // the sender is not dropped yet; nothing to do
            return;
        }
        // the sender is already dropped, so we need to clean up
        unsafe { self.common.deallocate() };
    }
}
