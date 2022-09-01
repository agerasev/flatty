use crate::{Flat, FlatBase};
use core::mem::{align_of, size_of};

/// Statically-sized flat type.
///
/// # Safety
///
/// `SIZE` must match `Self` size.
pub unsafe trait FlatSized: FlatBase + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

/// Dynamically-sized flat type.
///
/// For now it must be implemented for all [`Flat`](`crate::Flat`) types because there is no mutually exclusive traits in Rust yet.
///
/// # Safety
///
/// `AlignAs` type must have the same align as `Self`.
///
/// `ptr_metadata` must provide such value, that will give [`size()`](`FlatBase::size`)` <= `[`size_of_val()`](`core::mem::size_of_val`)` <= bytes.len()`.
pub unsafe trait FlatUnsized: FlatBase {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;

    /// Metadata to store in a wide pointer to `Self`.
    ///
    /// `None` is returned if type is `Sized`.
    fn ptr_metadata(bytes: &[u8]) -> Option<usize>;
}

unsafe impl<T: Flat + Sized> FlatSized for T {}

unsafe impl<T: FlatSized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;

    fn size(&self) -> usize {
        Self::SIZE
    }

    fn ptr_from_bytes(bytes: &[u8]) -> *const Self {
        bytes.as_ptr() as *const Self
    }
    fn ptr_from_mut_bytes(bytes: &mut [u8]) -> *mut Self {
        bytes.as_mut_ptr() as *mut Self
    }
}

unsafe impl<T: FlatSized> FlatUnsized for T {
    type AlignAs = T;

    fn ptr_metadata(_: &[u8]) -> Option<usize> {
        None
    }
}
