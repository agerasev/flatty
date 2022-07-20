use crate::base::{Flat, FlatBase};
use core::mem::size_of;

/// Statically-sized flat type.
pub trait FlatSized: FlatBase {
    /// Static size of the type.
    const SIZE: usize;
}

impl<T: Flat + Sized> FlatBase for T {
    type AlignAs = Self;

    const MIN_SIZE: usize = Self::SIZE;
    fn size(&self) -> usize {
        Self::SIZE
    }

    fn _ptr_metadata(_: &[u8]) -> usize {
        0
    }
}

impl<T: Flat + Sized> FlatSized for T {
    const SIZE: usize = size_of::<Self>();
}
