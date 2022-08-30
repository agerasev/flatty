use crate::{base::check_align_and_min_size, error::Error, FlatBase};
use core::slice::{from_raw_parts, from_raw_parts_mut};

/// Safe casting from and to bytes.
pub trait FlatCast: FlatBase {
    /// Checks that the pointer points to valid contents of the `Self`.
    ///
    /// # Safety
    ///
    /// `ptr` have proper alignment and data it points to have sufficient minimal size.
    ///
    /// This method returned `Ok` must guaratee that data pointed by `ptr` is in valid initialized state and we could safely dereference `ptr`.
    unsafe fn validate(ptr: *const Self) -> Result<(), Error>;

    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Error returned if:
    ///
    /// + Slice start address isn't properly aligned for `Self`.
    /// + Slice has insufficient size to store `Self` in a state described by data.
    /// + The [`Self::validate`] call returned an error.
    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        check_align_and_min_size::<Self>(bytes)?;
        let ptr = Self::ptr_from_bytes(bytes);
        unsafe {
            Self::validate(ptr)?;
            Ok(&*ptr)
        }
    }

    /// The same as [`Self::from_bytes`] but provides a mutable reference.
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<Self>(bytes)?;
        let ptr = Self::ptr_from_mut_bytes(bytes);
        unsafe {
            Self::validate(ptr)?;
            Ok(&mut *ptr)
        }
    }

    /// Binary representation of the `Self`.
    fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self as *const _ as *const u8, self.size()) }
    }

    /// Mutable binary representation of the `Self`.
    ///
    /// # Safety
    ///
    /// Modification of the slice contents must not make `Self` invalid.
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        from_raw_parts_mut(self as *mut _ as *mut u8, self.size())
    }
}
