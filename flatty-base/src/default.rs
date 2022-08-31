use crate::{base::check_align_and_min_size, error::Error, Flat, FlatCast};

pub trait FlatDefault: FlatCast {
    /// Initialize memory pointed by `ptr` into valid default state.
    ///
    /// # Safety
    ///
    /// `bytes` beginning must be aligned to [`FlatBase::ALIGN`] and its length must be greater or equal to [`FlatBase::MIN_SIZE`].
    ///
    /// This method returned `Ok` must guaratee that `bytes` could be safely transmuted to `Self`.
    unsafe fn default_contents(bytes: &mut [u8]) -> Result<(), Error>;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn default_in_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<Self>(bytes)?;
        unsafe { Self::default_contents(bytes) }?;
        let ptr = Self::ptr_from_mut_bytes(bytes);
        unsafe { Ok(&mut *ptr) }
    }
}

impl<T: Flat + Default + Sized> FlatDefault for T {
    unsafe fn default_contents(bytes: &mut [u8]) -> Result<(), Error> {
        *Self::ptr_from_mut_bytes(bytes) = Self::default();
        Ok(())
    }
}
