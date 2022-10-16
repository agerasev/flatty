use crate::{error::Error, mem::MaybeUninitUnsized, Flat};

/// In-place initializer of flat type.
///
/// # Safety
///
/// [`Self::init`] must be valid.
pub unsafe trait InplaceInitializer<T: Flat + ?Sized>: Sized {
    /// Apply initializer for uninitizalized memory.
    ///
    /// *In case of success must return reference to the same memory as `uninit`.*
    fn init(self, uninit: &mut MaybeUninitUnsized<T>) -> Result<&mut T, Error>;
}

/// # Safety
///
/// Methods must properly initialize memory.
pub trait FlatInit: Flat {
    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn placement_new<I: InplaceInitializer<Self>>(bytes: &mut [u8], initializer: I) -> Result<&mut Self, Error> {
        let this = MaybeUninitUnsized::from_mut_bytes(bytes)?;
        initializer.init(this)
    }

    fn placement_drop(&mut self) -> &mut [u8] {
        return unsafe { MaybeUninitUnsized::new_mut(self) }.as_mut_bytes();
    }

    fn placement_assign<I: InplaceInitializer<Self>>(&mut self, initializer: I) -> Result<&mut Self, Error> {
        Self::placement_new(self.placement_drop(), initializer)
    }
}

/// # Safety
///
/// Methods must properly initialize memory.
pub trait FlatDefault: FlatInit {
    type InplaceDefault: InplaceInitializer<Self>;

    /// Initialize uninitialized memory into valid default state.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn inplace_default() -> Self::InplaceDefault;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn placement_default(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Self::placement_new(bytes, Self::inplace_default())
    }
}

unsafe impl<T: Flat + Sized> InplaceInitializer<T> for T {
    fn init(self, uninit: &mut MaybeUninitUnsized<T>) -> Result<&mut T, Error> {
        Ok(uninit.as_mut_sized().write(self))
    }
}

impl<T: Flat + ?Sized> FlatInit for T {}

impl<T: FlatInit + Default + Sized> FlatDefault for T {
    type InplaceDefault = Self;

    fn inplace_default() -> Self::InplaceDefault {
        Self::default()
    }
}
