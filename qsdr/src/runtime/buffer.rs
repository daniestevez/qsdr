mod cache_aligned;
pub use cache_aligned::CacheAlignedBuffer;

/// Buffer.
///
/// This trait represents a buffer that can be used by qsdr.
///
/// # Safety
///
/// The `as_mut_ptr` method must return a pointer to a contiguous allocation of
/// `len` items of type `Item` that is correctly aligned and padded for this
/// type. Read and write accesses to this allocation should be valid as long as
/// the `Buffer` object is alive.
///
/// Multiple calls to `as_mut_ptr` and `len` should always return the same
/// value.
#[allow(clippy::len_without_is_empty)]
pub unsafe trait Buffer: Send {
    type Item;

    fn as_mut_ptr(&self) -> *mut Self::Item;

    fn len(&self) -> usize;
}

mod assert {
    #![allow(dead_code)]
    use super::*;

    // compile time assert for object safety of buffer
    fn buffer_is_object_safe<T>(_: &dyn Buffer<Item = T>) {}
}
