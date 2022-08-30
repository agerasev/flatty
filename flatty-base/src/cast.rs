use crate::{base::check_size_and_align, error::Error, FlatBase};
use core::slice::{from_raw_parts, from_raw_parts_mut};

/// Safe casting from and to bytes.
pub trait FlatCast: FlatBase {
    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Error returned if:
    ///
    /// + Memory isn't properly aligned for [`Self`].
    /// + Memory has insufficient size to store [`Self`].
    /// + The data describes invalid state of [`Self`].
    fn from_bytes(mem: &[u8]) -> Result<&Self, Error> {
        check_size_and_align::<Self>(mem)?;
        Ok(unsafe { Self::from_bytes_unchecked(mem) })
    }

    /// The same as [`from_bytes`](`Self::from_bytes`) but provides a mutable reference.
    fn from_mut_bytes(mem: &mut [u8]) -> Result<&mut Self, Error> {
        check_size_and_align::<Self>(mem)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(mem) })
    }

    /// Interpret without checks.
    ///
    /// # Safety
    ///
    /// Memory must have suitable size and align for `Self` and its contents must be valid.  
    unsafe fn from_bytes_unchecked(mem: &[u8]) -> &Self;

    /// Interpret without checks providing mutable reference.
    ///
    /// # Safety
    ///
    /// Memory must have suitable size and align for `Self` and its contents must be valid.  
    unsafe fn from_mut_bytes_unchecked(mem: &mut [u8]) -> &mut Self;

    /// Binary representation of the [`Self`].
    fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self as *const _ as *const u8, self.size()) }
    }

    /// Mutable binary representation of the [`Self`].
    ///
    /// # Safety
    ///
    /// Modification of the slice contents must not make [`Self`] invalid.
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        from_raw_parts_mut(self as *mut _ as *mut u8, self.size())
    }
}
