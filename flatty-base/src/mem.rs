use crate::{Error, ErrorKind, FlatSized, FlatUnsized};
use core::{
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Check that memory size and alignment are suitable for `Self`.
fn check_align_and_min_size<T: FlatUnsized + ?Sized>(mem: &[u8]) -> Result<(), Error> {
    if mem.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })
    } else if mem.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign,
            pos: 0,
        })
    } else {
        Ok(())
    }
}

/// Maybe uninit unsized.
///
/// Like [`MaybeUninit`](`core::mem::MaybeUninit`) but for `?Sized` types.
#[repr(C)]
pub struct Muu<T: FlatUnsized + ?Sized> {
    _align: [T::AlignAs; 0],
    bytes: [u8],
}

impl<T: FlatUnsized + ?Sized> Muu<T> {
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const [u8] as *const Muu<T>)
    }
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes as *mut [u8] as *mut Muu<T>)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }
    pub fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(bytes) })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    pub fn as_ptr(&self) -> *const T {
        T::ptr_from_bytes(self.as_bytes())
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        T::ptr_from_mut_bytes(self.as_mut_bytes())
    }
}
impl<T: FlatSized> Muu<T> {
    pub fn from_sized(mu: &MaybeUninit<T>) -> &Self {
        let bytes = unsafe { from_raw_parts(mu.as_ptr() as *const u8, T::SIZE) };
        unsafe { Self::from_bytes_unchecked(bytes) }
    }
    pub fn from_mut_sized(mu: &mut MaybeUninit<T>) -> &mut Self {
        let bytes = unsafe { from_raw_parts_mut(mu.as_mut_ptr() as *mut u8, T::SIZE) };
        unsafe { Self::from_mut_bytes_unchecked(bytes) }
    }
}

/// Assume that slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub(crate) unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const [MaybeUninit<T>] as *const [T])
}

/// Assume that mutable slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub(crate) unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}
