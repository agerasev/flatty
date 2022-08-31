use crate::{base::check_align_and_min_size, error::Error, FlatBase};
use core::slice::{from_raw_parts, from_raw_parts_mut};

/// Safe casting from and to bytes.
pub trait FlatCast: FlatBase {
    /// Check that `bytes` contents is valid for `Self`.
    ///
    /// # Safety
    ///
    /// `bytes` beginning must be aligned to [`FlatBase::ALIGN`] and its length must be greater or equal to [`FlatBase::MIN_SIZE`].
    unsafe fn validate_contents(bytes: &[u8]) -> Result<(), Error>;

    /// Check that `bytes` is a valid `Self` representation.
    ///
    /// This method returned `Ok` must guaratee that `bytes` could be safely transmuted to `Self`.
    fn validate_bytes(bytes: &[u8]) -> Result<(), Error> {
        check_align_and_min_size::<Self>(bytes)?;
        unsafe { Self::validate_contents(bytes) }?;
        Ok(())
    }

    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Error returned if:
    ///
    /// + Slice start address isn't properly aligned for `Self`.
    /// + Slice has insufficient size to store `Self` in a state described by data.
    /// + The [`Self::validate_bytes`] returned an error.
    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        Self::validate_bytes(bytes)?;
        let ptr = Self::ptr_from_bytes(bytes);
        unsafe { Ok(&*ptr) }
    }

    /// The same as [`Self::from_bytes`] but provides a mutable reference.
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Self::validate_bytes(bytes)?;
        let ptr = Self::ptr_from_mut_bytes(bytes);
        unsafe { Ok(&mut *ptr) }
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
