//! # Flatty
//!
//! ## TODO
//!
//! + Should we allow [`FlatVec`] items to be non-[`Copy`]?
//! + What if constructed from non-zeroed slice? Should we validate on constructing?

use std::mem::{align_of, size_of};

mod prim;
mod util;

pub mod len;
pub mod unsized_enum;
pub mod vec;

pub use len::FlatLen;
pub use unsized_enum::UnsizedEnum;
pub use vec::FlatVec;

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
    /// # Safety
    unsafe fn from_slice(mem: &[u8]) -> &Self;
    /// # Safety
    unsafe fn from_slice_mut(mem: &mut [u8]) -> &mut Self;
}

unsafe impl<T: FlatSized> Flat for T {
    type AlignAs = Self;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

impl<T: FlatSized> FlatExt for T {
    unsafe fn from_slice(mem: &[u8]) -> &Self {
        assert!(mem.len() >= Self::SIZE);
        assert!(mem.as_ptr().align_offset(Self::ALIGN) == 0);

        let ptr = mem.as_ptr() as *const Self;
        &*ptr
    }
    unsafe fn from_slice_mut(mem: &mut [u8]) -> &mut Self {
        assert!(mem.len() >= Self::SIZE);
        assert!(mem.as_ptr().align_offset(Self::ALIGN) == 0);

        let ptr = mem.as_mut_ptr() as *mut Self;
        &mut *ptr
    }
}
