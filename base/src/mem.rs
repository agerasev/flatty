use crate::{utils::floor_mul, Emplacer, Error, ErrorKind, FlatDefault, FlatSized, FlatUnsized, FlatValidate};
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

#[doc(hidden)]
pub unsafe trait AsBytes {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self;
    unsafe fn from_mut_bytes(bytes: &mut [u8]) -> &mut Self;
    unsafe fn as_bytes(&self) -> &[u8];
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8];
}
unsafe impl AsBytes for [u8] {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        bytes
    }
    unsafe fn from_mut_bytes(bytes: &mut [u8]) -> &mut Self {
        bytes
    }
    unsafe fn as_bytes(&self) -> &[u8] {
        self
    }
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        self
    }
}
unsafe impl<T: FlatSized> AsBytes for MaybeUninit<T> {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
    unsafe fn from_mut_bytes(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) }
    }
    unsafe fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self.as_ptr() as *const u8, T::SIZE) }
    }
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.as_mut_ptr() as *mut u8, T::SIZE) }
    }
}

/// Memory that can be reinterpreted as `T` but its contents may be invalid for `T`.
///
/// Guarantees that underlying memory properly aligned and have enough length to store `T`.
#[repr(C)]
pub struct Unvalidated<T: FlatUnsized + ?Sized> {
    align: [T::AlignAs; 0],
    bytes: T::AsBytes,
}

impl<T: FlatUnsized + ?Sized> Unvalidated<T> {
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        let bytes = bytes.get_unchecked(..floor_mul(bytes.len(), T::ALIGN));
        &*(T::AsBytes::from_bytes(bytes) as *const T::AsBytes as *const Self)
    }
    /// # Safety
    ///
    /// Bytes must be aligned to `T::ALIGN` and have length greater or equal to `T::MIN_SIZE`.
    pub unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut Self {
        let bytes = bytes.get_unchecked_mut(..floor_mul(bytes.len(), T::ALIGN));
        &mut *(T::AsBytes::from_mut_bytes(bytes) as *mut T::AsBytes as *mut Self)
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
        unsafe { self.bytes.as_bytes() }
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { self.bytes.as_mut_bytes() }
    }

    pub fn new_in_place<I: Emplacer<T>>(&mut self, emplacer: I) -> Result<&mut T, Error> {
        T::new_in_place(self, emplacer)
    }
}

impl<T: FlatValidate + ?Sized> Unvalidated<T> {
    pub fn validate(&self) -> Result<&T, Error> {
        T::validate(self)
    }
    pub fn validate_mut(&mut self) -> Result<&mut T, Error> {
        T::validate_mut(self)
    }
}

impl<T: FlatDefault + ?Sized> Unvalidated<T> {
    pub fn default_in_place(&mut self) -> Result<&mut T, Error> {
        T::default_in_place(self)
    }
}

impl<T: FlatSized> Unvalidated<T> {
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
