use crate::{base::check_align_and_min_size, Error, Flat, FlatCast};

pub trait FlatDefault: FlatCast {
    /// Initialize memory pointed by `ptr` into valid default state.
    ///
    /// # Safety
    ///
    /// `ptr` have proper alignment and data it points to have sufficient minimal size.
    ///
    /// The method returned `Ok` must guarantee that data pointed by `ptr` must have valid initialized state.
    unsafe fn init_default(ptr: *mut Self) -> Result<(), Error>;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn placement_default(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        check_align_and_min_size::<Self>(bytes)?;
        unsafe {
            let ptr = Self::ptr_from_mut_bytes(bytes);
            Self::init_default(ptr)?;
            Ok(&mut *ptr)
        }
    }
}

impl<T: Flat + Default + Sized> FlatDefault for T {
    unsafe fn init_default(ptr: *mut Self) -> Result<(), Error> {
        *ptr = Self::default();
        Ok(())
    }
}
