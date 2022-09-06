use crate::{mem::MaybeUninitUnsized, Flat, FlatBase, FlatMaybeUnsized};
use core::{
    mem::{align_of, size_of},
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Statically-sized flat type.
///
/// # Safety
///
/// `SIZE` must match `Self` size.
pub unsafe trait FlatSized: FlatBase + FlatMaybeUnsized + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

unsafe impl<T: Flat + Sized> FlatSized for T {}

unsafe impl<T: FlatSized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

unsafe impl<T: FlatSized> FlatMaybeUnsized for T {
    type AlignAs = T;

    fn ptr_metadata(_this: &MaybeUninitUnsized<Self>) -> usize {
        panic!("Sized type ptr has no metadata");
    }
    unsafe fn bytes_len(_this: *const Self) -> usize {
        Self::SIZE
    }

    fn ptr_from_uninit(this: &MaybeUninitUnsized<Self>) -> *const Self {
        this.as_bytes().as_ptr() as *const Self
    }
    fn ptr_from_mut_uninit(this: &mut MaybeUninitUnsized<Self>) -> *mut Self {
        this.as_mut_bytes().as_mut_ptr() as *mut Self
    }

    unsafe fn ptr_to_uninit<'a>(this: *const Self) -> &'a MaybeUninitUnsized<Self> {
        MaybeUninitUnsized::from_bytes_unchecked(from_raw_parts(this as *const u8, Self::SIZE))
    }
    unsafe fn ptr_to_mut_uninit<'a>(this: *mut Self) -> &'a mut MaybeUninitUnsized<Self> {
        MaybeUninitUnsized::from_mut_bytes_unchecked(from_raw_parts_mut(
            this as *mut u8,
            Self::SIZE,
        ))
    }
}
