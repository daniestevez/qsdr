use super::buffer::Buffer;
use std::{
    fmt,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

pub struct Sheet<B: Buffer> {
    buffer: B,
    text: NonNull<B::Item>,
    text_len: usize,
}

impl<B: Buffer> fmt::Debug for Sheet<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Sheet")
            .field("buffer.as_mut_ptr", &self.buffer.as_mut_ptr())
            .field("buffer.len", &self.buffer.len())
            .field("text", &self.text)
            .field("text_len", &self.text_len)
            .finish()
    }
}

unsafe impl<B> Send for Sheet<B>
where
    B: Buffer + Send,
    B::Item: Send,
{
}

unsafe impl<B> Sync for Sheet<B>
where
    B: Buffer + Sync,
    B::Item: Sync,
{
}

impl<B: Buffer> Sheet<B> {
    pub fn new(buffer: B) -> Sheet<B> {
        let text = NonNull::new(buffer.as_mut_ptr()).unwrap();
        let text_len = buffer.len();
        Sheet {
            buffer,
            text,
            text_len,
        }
    }

    pub fn left_margin_len(&self) -> usize {
        unsafe { self.text.as_ptr().offset_from(self.buffer.as_mut_ptr()) as usize }
    }

    pub fn right_margin_len(&self) -> usize {
        self.buffer.len() - self.text_len - self.left_margin_len()
    }

    pub fn set_margins(&mut self, left_margin_len: usize, right_margin_len: usize) {
        let buffer_len = self.buffer.len();
        assert!(left_margin_len + right_margin_len <= buffer_len);
        self.text = unsafe {
            NonNull::new(self.buffer.as_mut_ptr())
                .unwrap()
                .add(left_margin_len)
        };
        self.text_len = buffer_len - left_margin_len - right_margin_len;
    }

    pub fn len(&self) -> usize {
        self.text_len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn extend_left(&mut self, len: usize) {
        assert!(len <= self.left_margin_len());
        self.text = unsafe { self.text.offset(-(len as isize)) };
    }

    pub fn extend_right(&mut self, len: usize) {
        assert!(len <= self.right_margin_len());
        self.text_len += len;
    }

    pub fn shrink_left(&mut self, len: usize) {
        assert!(len <= self.len());
        self.text = unsafe { self.text.add(len) };
        self.text_len -= len;
    }

    pub fn shrink_right(&mut self, len: usize) {
        assert!(len <= self.len());
        self.text_len -= len;
    }
}

impl<B: Buffer> Deref for Sheet<B> {
    type Target = [B::Item];

    fn deref(&self) -> &[B::Item] {
        unsafe { slice::from_raw_parts(self.text.as_ptr().cast_const(), self.text_len) }
    }
}

impl<B: Buffer> DerefMut for Sheet<B> {
    fn deref_mut(&mut self) -> &mut [B::Item] {
        unsafe { slice::from_raw_parts_mut(self.text.as_ptr(), self.text_len) }
    }
}
