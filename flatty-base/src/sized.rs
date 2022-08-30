use crate::{Flat, FlatBase};
use core::mem::{align_of, size_of};

/// Statically-sized flat type.
pub trait FlatSized: Flat + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

/// Dynamically-sized flat type.
pub trait FlatUnsized: Flat {
    /// Sized type that has the same alignment as [`Self`].
    type AlignAs: Sized;

    /// Metadata to store in a wide pointer to [`Self`].
    fn ptr_metadata(mem: &[u8]) -> usize;
}

impl<T: Flat + Sized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

impl<T: Flat + Sized> FlatSized for T {}
