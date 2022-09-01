use crate::{error::Error, mem::Muu, FlatUnsized};
use core::slice::{from_raw_parts, from_raw_parts_mut};

/// Safe casting from and to bytes.
pub trait FlatCast: FlatUnsized {
    /// Check that `bytes` is a valid `Self` representation.
    ///
    /// This method returned `Ok` must guaratee that `bytes` could be safely transmuted to `Self`.
    fn validate(this: &Muu<Self>) -> Result<(), Error>;

    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Error returned if:
    ///
    /// + Slice start address isn't properly aligned for `Self`.
    /// + Slice has insufficient size to store `Self` in a state described by data.
    /// + The [`Self::validate`] returned an error.
    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        let this = Muu::from_bytes(bytes)?;
        Self::validate(this)?;
        unsafe { Ok(&*this.as_ptr()) }
    }

    /// The same as [`Self::from_bytes`] but provides a mutable reference.
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        let this = Muu::from_mut_bytes(bytes)?;
        Self::validate(this)?;
        Ok(unsafe { &mut *this.as_mut_ptr() })
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
