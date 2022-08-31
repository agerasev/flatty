use crate::{Flat, FlatBase};
use core::mem::{align_of, size_of};

/// Statically-sized flat type.
///
/// # Safety
///
/// Must not be implemented at the same time as [`FlatUnsized`].
pub unsafe trait FlatSized: Flat + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

/// Dynamically-sized flat type.
///
/// # Safety
///
/// Must not be implemented at the same time as [`FlatSized`].
pub unsafe trait FlatUnsized: Flat {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;

    /// Metadata to store in a wide pointer to `Self`.
    fn ptr_metadata(bytes: &[u8]) -> usize;
}

impl<T: Flat + Sized> FlatBase for T {
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

unsafe impl<T: Flat + Sized> FlatSized for T {}
