use crate::{error::Error, mem::MaybeUninitUnsized, Flat};

/// # Safety
///
/// Methods must properly initialize memory.
pub unsafe trait FlatDefault: Flat {
    /// Initialize memory pointed by `ptr` into valid default state.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn init_default(this: &mut MaybeUninitUnsized<Self>) -> Result<(), Error>;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn placement_default(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        let this = MaybeUninitUnsized::from_bytes_mut(bytes)?;
        Self::init_default(this)?;
        Ok(unsafe { this.assume_init_mut() })
    }
}

unsafe impl<T: Flat + Default + Sized> FlatDefault for T {
    fn init_default(this: &mut MaybeUninitUnsized<Self>) -> Result<(), Error> {
        unsafe { *this.assume_init_mut() = Self::default() };
        Ok(())
    }
}
