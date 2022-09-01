use crate::{error::Error, mem::Muu, Flat};

pub trait FlatDefault: Flat {
    /// Initialize memory pointed by `ptr` into valid default state.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn init_default(this: &mut Muu<Self>) -> Result<(), Error>;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn placement_default(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        let this = Muu::from_mut_bytes(bytes)?;
        Self::init_default(this)?;
        unsafe { Ok(&mut *this.as_mut_ptr()) }
    }
}

impl<T: Flat + Default + Sized> FlatDefault for T {
    fn init_default(this: &mut Muu<Self>) -> Result<(), Error> {
        unsafe { *this.as_mut_ptr() = Self::default() };
        Ok(())
    }
}
