use crate::{Emplacer, Error, ErrorKind, FlatCheck, FlatDefault, FlatSized, FlatUnsized};
use core::{
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Check that memory size and alignment are suitable for `Self`.
fn check_align_and_min_size<T: FlatUnsized + ?Sized>(mem: &[u8]) -> Result<(), Error> {
    if mem.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign,
            pos: 0,
        })
    } else if mem.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })
    } else {
        Ok(())
    }
}

/// Maybe uninit unsized.
///
/// Like [`MaybeUninit`](`core::mem::MaybeUninit`) but also for `?Sized` types.
#[repr(C)]
pub struct MaybeUninitUnsized<T: FlatUnsized + ?Sized> {
    _align: [T::AlignAs; 0],
    bytes: [u8],
}

impl<T: FlatUnsized + ?Sized> MaybeUninitUnsized<T> {
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const [u8] as *const Self)
    }
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes as *mut [u8] as *mut Self)
    }
    /// Try to convert bytes to potentially unintialized instance of `T`.
    ///
    /// Error returned if:
    ///
    /// + Slice start address isn't properly aligned for `T`.
    /// + Slice has insufficient size to store `T` even it has minimally possible size.
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }
    /// The same as [`Self::from_bytes`] but returns mutable reference.
    pub fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<T>(bytes)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(bytes) })
    }
    /// # Safety
    ///
    /// `self` must be initialized.
    pub unsafe fn assume_init(&self) -> &T {
        T::from_uninit_unchecked(self)
    }
    /// # Safety
    ///
    /// `self` must be initialized.
    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        T::from_mut_uninit_unchecked(self)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub fn new_in_place<I: Emplacer<T>>(&mut self, emplacer: I) -> Result<&mut T, Error> {
        T::new_in_place(self, emplacer)
    }
}

impl<T: FlatCheck + ?Sized> MaybeUninitUnsized<T> {
    pub fn validate(&self) -> Result<&T, Error> {
        T::validate(self)
    }
    pub fn validate_mut(&mut self) -> Result<&mut T, Error> {
        T::validate_mut(self)
    }
}

impl<T: FlatDefault + ?Sized> MaybeUninitUnsized<T> {
    pub fn default_in_place(&mut self) -> Result<&mut T, Error> {
        T::default_in_place(self)
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

    pub fn as_sized(&self) -> &MaybeUninit<T> {
        unsafe { &*(self.as_bytes().as_ptr() as *const MaybeUninit<T>) }
    }
    pub fn as_mut_sized(&mut self) -> &mut MaybeUninit<T> {
        unsafe { &mut *(self.as_mut_bytes().as_mut_ptr() as *mut MaybeUninit<T>) }
    }
}
