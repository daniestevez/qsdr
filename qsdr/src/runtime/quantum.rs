use super::{buffer::Buffer, sheet::Sheet};

#[derive(Debug)]
pub struct Quantum<B: Buffer> {
    sheet: Sheet<B>,
}

impl<B: Buffer> Quantum<B> {
    pub fn new(buffer: B) -> Quantum<B> {
        Quantum {
            sheet: Sheet::new(buffer),
        }
    }

    pub fn as_slice(&self) -> &[B::Item] {
        &self.sheet
    }

    pub fn as_mut_slice(&mut self) -> &mut [B::Item] {
        &mut self.sheet
    }

    pub fn left_margin_len(&self) -> usize {
        self.sheet.left_margin_len()
    }

    pub fn right_margin_len(&self) -> usize {
        self.sheet.right_margin_len()
    }

    pub fn set_margins(&mut self, left_margin_len: usize, right_margin_len: usize) {
        self.sheet.set_margins(left_margin_len, right_margin_len);
    }

    pub fn len(&self) -> usize {
        self.sheet.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sheet.is_empty()
    }

    pub fn extend_left(&mut self, len: usize) {
        self.sheet.extend_left(len);
    }

    pub fn extend_right(&mut self, len: usize) {
        self.sheet.extend_right(len);
    }

    pub fn shrink_left(&mut self, len: usize) {
        self.sheet.shrink_left(len);
    }

    pub fn shrink_right(&mut self, len: usize) {
        self.sheet.shrink_right(len);
    }
}

impl<B> Quantum<B>
where
    B: Buffer,
    B::Item: Clone,
{
    pub fn snapshot(&self) -> QuantumSnapshot<B::Item> {
        QuantumSnapshot {
            slice: Box::from(self.as_slice()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuantumSnapshot<T> {
    slice: Box<[T]>,
}

impl<T> QuantumSnapshot<T> {
    pub fn as_slice(&self) -> &[T] {
        &self.slice
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.slice
    }
}

impl<T: Clone> From<&[T]> for QuantumSnapshot<T> {
    fn from(slice: &[T]) -> QuantumSnapshot<T> {
        QuantumSnapshot {
            slice: Box::from(slice),
        }
    }
}

impl<T> From<Vec<T>> for QuantumSnapshot<T> {
    fn from(vec: Vec<T>) -> QuantumSnapshot<T> {
        QuantumSnapshot {
            slice: vec.into_boxed_slice(),
        }
    }
}
