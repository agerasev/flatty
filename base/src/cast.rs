use crate::{error::Error, mem::MaybeUninitUnsized, FlatMaybeUnsized};

/// Safe casting from and to bytes.
pub trait FlatCast: FlatMaybeUnsized {
    /// Check that `bytes` is a valid `Self` representation.
    ///
    /// This method returned `Ok` must guaratee that `bytes` could be safely transmuted to `Self`.
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error>;

    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Error returned if:
    ///
    /// + Slice start address isn't properly aligned for `Self`.
    /// + Slice has insufficient size to store `Self` in a state described by data.
    /// + The [`Self::validate`] returned an error.
    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        let this = MaybeUninitUnsized::from_bytes(bytes)?;
        Self::validate(this)?;
        Ok(unsafe { this.assume_init() })
    }
    /// The same as [`Self::from_bytes`] but provides a mutable reference.
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        let this = MaybeUninitUnsized::from_bytes_mut(bytes)?;
        Self::validate(this)?;
        Ok(unsafe { this.assume_init_mut() })
    }

    /// Binary representation of the `Self`.
    fn as_bytes(&self) -> &[u8] {
        self.to_uninit().as_bytes()
    }
    /// Mutable binary representation of the `Self`.
    ///
    /// # Safety
    ///
    /// Modification of the slice contents must not make `Self` invalid.
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.to_uninit_mut().as_bytes_mut()
    }
}
