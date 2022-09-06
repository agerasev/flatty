use crate::{Error, ErrorKind, FlatMaybeUnsized, FlatSized};
use core::{
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Check that memory size and alignment are suitable for `Self`.
fn check_align_and_min_size<T: FlatMaybeUnsized + ?Sized>(mem: &[u8]) -> Result<(), Error> {
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
pub struct MaybeUninitUnsized<T: FlatMaybeUnsized + ?Sized> {
    _align: [T::AlignAs; 0],
    bytes: [u8],
}

impl<T: FlatMaybeUnsized + ?Sized> MaybeUninitUnsized<T> {
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const [u8] as *const MaybeUninitUnsized<T>)
    }
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes as *mut [u8] as *mut MaybeUninitUnsized<T>)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }
    pub fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(bytes) })
    }
    /// # Safety
    ///
    /// `this` must point to existing data which is aligned and of sufficient size.
    pub unsafe fn from_ptr<'a>(this: *const T) -> &'a Self {
        T::ptr_to_uninit(this)
    }
    /// # Safety
    ///
    /// `this` must point to existing data which is aligned and of sufficient size.
    pub unsafe fn from_mut_ptr<'a>(this: *mut T) -> &'a mut Self {
        T::ptr_to_mut_uninit(this)
    }

    /// # Safety
    ///
    /// `self` must be initialized.
    pub unsafe fn assume_init_ref(&self) -> &T {
        T::from_uninit_unchecked(self)
    }
    /// # Safety
    ///
    /// `self` must be initialized.
    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        T::from_mut_uninit_unchecked(self)
    }
    pub fn as_ptr(&self) -> *const T {
        T::ptr_from_uninit(self)
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        T::ptr_from_mut_uninit(self)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl<T: FlatSized> MaybeUninitUnsized<T> {
    pub fn from_sized(mu: &MaybeUninit<T>) -> &Self {
        let bytes = unsafe { from_raw_parts(mu.as_ptr() as *const u8, T::SIZE) };
        unsafe { Self::from_bytes_unchecked(bytes) }
    }
    pub fn from_mut_sized(mu: &mut MaybeUninit<T>) -> &mut Self {
        let bytes = unsafe { from_raw_parts_mut(mu.as_mut_ptr() as *mut u8, T::SIZE) };
        unsafe { Self::from_mut_bytes_unchecked(bytes) }
    }
}
