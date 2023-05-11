use crate::{mem::Unvalidated, Emplacer, Error, Flat, FlatBase, FlatUnsized};
use core::{
    mem::{align_of, size_of, MaybeUninit},
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
    type AsBytes = MaybeUninit<T>;

    fn ptr_metadata(_this: &Unvalidated<Self>) -> usize {
        panic!("Sized type ptr has no metadata");
    }
    fn bytes_len(_this: *const Self) -> usize {
        Self::SIZE
    }

    fn ptr_from_uninit(this: &Unvalidated<Self>) -> *const Self {
        this.as_bytes().as_ptr() as *const Self
    }
    fn ptr_from_mut_uninit(this: &mut Unvalidated<Self>) -> *mut Self {
        this.as_mut_bytes().as_mut_ptr() as *mut Self
    }

    unsafe fn ptr_as_uninit<'a>(this: *const Self) -> &'a Unvalidated<Self> {
        unsafe { Unvalidated::from_bytes_unchecked(from_raw_parts(this as *const u8, Self::SIZE)) }
    }
    unsafe fn ptr_as_mut_uninit<'a>(this: *mut Self) -> &'a mut Unvalidated<Self> {
        Unvalidated::from_mut_bytes_unchecked(from_raw_parts_mut(this as *mut u8, Self::SIZE))
    }
}

impl<T: Flat> Emplacer<T> for T {
    fn emplace(self, unval: &mut Unvalidated<T>) -> Result<&mut T, Error> {
        unsafe {
            unval.as_mut_ptr().write(self);
            Ok(unval.assume_init_mut())
        }
    }
}
