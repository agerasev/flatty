use crate::base::{Flat, FlatBase, FlatUnsized};
use core::mem::{align_of, size_of};

/// Statically-sized flat type.
pub trait FlatSized: FlatBase {
    /// Static size of the type.
    const SIZE: usize;
}

impl<T: Flat + Sized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;
    fn size(&self) -> usize {
        Self::SIZE
    }
}

impl<T: Flat + Sized> FlatSized for T {
    const SIZE: usize = size_of::<Self>();
}

impl<T: Flat + Sized> FlatUnsized for T {
    type AlignAs = Self;

    fn ptr_metadata(_: &[u8]) -> usize {
        panic!("Getting ptr_metadata from sized type");
    }
}
