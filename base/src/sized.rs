use crate::{mem::MaybeUninitUnsized, Emplacer, Error, Flat, FlatBase, FlatUnsized};
use core::{
    mem::{align_of, size_of},
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Statically-sized flat type.
///
/// # Safety
///
/// `SIZE` must match `Self` size.
pub unsafe trait FlatSized: FlatUnsized + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

unsafe impl<T: Flat> FlatSized for T {}

unsafe impl<T: FlatSized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

unsafe impl<T: FlatSized> FlatUnsized for T {
    type AlignAs = T;

    fn ptr_metadata(_this: &MaybeUninitUnsized<Self>) -> usize {
        panic!("Sized type ptr has no metadata");
    }
    fn bytes_len(_this: &Self) -> usize {
        Self::SIZE
    }

    unsafe fn from_uninit_unchecked(this: &MaybeUninitUnsized<Self>) -> &Self {
        &*(this.as_bytes().as_ptr() as *const Self)
    }
    unsafe fn from_mut_uninit_unchecked(this: &mut MaybeUninitUnsized<Self>) -> &mut Self {
        &mut *(this.as_mut_bytes().as_mut_ptr() as *mut Self)
    }

    fn as_uninit(&self) -> &MaybeUninitUnsized<Self> {
        unsafe { MaybeUninitUnsized::from_bytes_unchecked(from_raw_parts(self as *const _ as *const u8, Self::SIZE)) }
    }
    unsafe fn as_mut_uninit(&mut self) -> &mut MaybeUninitUnsized<Self> {
        MaybeUninitUnsized::from_mut_bytes_unchecked(from_raw_parts_mut(self as *mut _ as *mut u8, Self::SIZE))
    }
}

unsafe impl<T: Flat> Emplacer<T> for T {
    fn emplace(self, uninit: &mut MaybeUninitUnsized<T>) -> Result<&mut T, Error> {
        Ok(uninit.as_mut_sized().write(self))
    }
}
