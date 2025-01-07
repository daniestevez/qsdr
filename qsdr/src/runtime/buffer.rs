mod cache_aligned;
pub use cache_aligned::CacheAlignedBuffer;

// TODO: write safety docs
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
