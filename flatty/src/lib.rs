use std::mem::{align_of, size_of};

mod len;
mod prim;
mod vec;

pub use len::*;
pub use prim::*;
pub use vec::*;

/// # Safety
pub unsafe trait Flat {
    /// Sized type that has the same alignment as [`Self`].
    type AlignAs: Sized;
    const ALIGN: usize = align_of::<Self::AlignAs>();

    fn size(&self) -> usize;
}

/// # Safety
pub unsafe trait FlatSized: Flat + Sized {
    const SIZE: usize = size_of::<Self>();
}

pub trait FlatExt {
    fn from_slice(mem: &[u8]) -> &Self;
    fn from_slice_mut(mem: &mut [u8]) -> &mut Self;
}

unsafe impl<T: FlatSized> Flat for T {
    type AlignAs = Self;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

impl<T: FlatSized> FlatExt for T {
    fn from_slice(mem: &[u8]) -> &Self {
        assert!(mem.len() >= Self::SIZE);
        assert!(mem.as_ptr().align_offset(Self::ALIGN) == 0);

        let ptr = mem.as_ptr() as *const Self;
        unsafe { &*ptr }
    }
    fn from_slice_mut(mem: &mut [u8]) -> &mut Self {
        assert!(mem.len() >= Self::SIZE);
        assert!(mem.as_ptr().align_offset(Self::ALIGN) == 0);

        let ptr = mem.as_mut_ptr() as *mut Self;
        unsafe { &mut *ptr }
    }
}
