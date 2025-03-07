use super::Buffer;
use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

// The data cache line size is 64 bytes in most CPUs.
pub const CACHE_LINE_SIZE: usize = 64;

#[derive(Debug)]
pub struct CacheAlignedBuffer<T> {
    ptr: NonNull<T>,
    len: usize,
}

unsafe impl<T: Send> Send for CacheAlignedBuffer<T> {}
unsafe impl<T: Sync> Sync for CacheAlignedBuffer<T> {}

impl<T> CacheAlignedBuffer<T> {
    fn layout(len: usize) -> Layout {
        Layout::array::<T>(len)
            .unwrap()
            .align_to(CACHE_LINE_SIZE)
            .unwrap()
    }
}

impl<T: Default> CacheAlignedBuffer<T> {
    pub fn new(len: usize) -> CacheAlignedBuffer<T> {
        CacheAlignedBuffer::from_fn(len, |_| T::default())
    }
}

impl<T: Clone> CacheAlignedBuffer<T> {
    pub fn from_value(len: usize, value: T) -> CacheAlignedBuffer<T> {
        CacheAlignedBuffer::from_fn(len, |_| value.clone())
    }
}

impl<T> CacheAlignedBuffer<T> {
    pub fn from_fn<F>(len: usize, mut f: F) -> CacheAlignedBuffer<T>
    where
        F: FnMut(usize) -> T,
    {
        if size_of::<T>() == 0 || len == 0 {
            let buffer = CacheAlignedBuffer {
                ptr: NonNull::dangling(),
                len,
            };
            if size_of::<T>() == 0 {
                // Create ZST objects contained in the buffer.
                for n in 0..len {
                    // SAFETY: this is a write of a ZST to a dangling pointer
                    unsafe {
                        std::ptr::write(buffer.ptr.as_ptr(), f(n));
                    }
                }
            }
            return buffer;
        }

        let layout = Self::layout(len);
        // SAFETY: the layout is generated for a non-zero size
        let ptr = unsafe { alloc(layout) };
        let Some(ptr) = NonNull::new(ptr.cast::<T>()) else {
            handle_alloc_error(layout);
        };
        for n in 0..len {
            // SAFETY: the pointer is in-bounds of an allocated object that is
            // appropriately aligned for T
            unsafe {
                std::ptr::write(ptr.as_ptr().add(n), f(n));
            }
        }
        CacheAlignedBuffer { ptr, len }
    }
}

impl<T> Drop for CacheAlignedBuffer<T> {
    fn drop(&mut self) {
        // First drop each element contained in the buffer; This even needs to
        // be done for ZSTs, since their Drop implementation could have
        // side-effects.
        for n in 0..self.len {
            // SAFETY: if T is not ZST, this drops in place using a valid
            // pointer to an object of type T that is initialized. If T is ZST,
            // the pointer is always the same dangling pointer and we have
            // initialized as many objects of type T as we are dropping here.
            unsafe {
                std::ptr::drop_in_place(self.ptr.as_ptr().add(n));
            }
        }

        if size_of::<T>() == 0 || self.len == 0 {
            // no allocation was done, so there is no need to deallocate
            return;
        }

        // SAFETY: self.ptr was allocated with the same allocator and layout
        unsafe {
            dealloc(self.ptr.as_ptr().cast::<u8>(), Self::layout(self.len));
        }
    }
}

impl<T> Deref for CacheAlignedBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        // SAFETY: self.ptr either points to an allocation of the correct length
        // containing initialized data or is a dangling pointer and self.len is
        // zero or T is ZST
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().cast_const(), self.len) }
    }
}

impl<T> DerefMut for CacheAlignedBuffer<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: self.ptr either points to an allocation of the correct length
        // containing initialized data or is a dangling pointer and self.len is
        // zero or T is ZST
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

unsafe impl<T: Send> Buffer for CacheAlignedBuffer<T> {
    type Item = T;

    fn as_mut_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    fn len(&self) -> usize {
        self.len
    }
}
