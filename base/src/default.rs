use crate::{error::Error, mem::MaybeUninitUnsized, Emplacer, Flat};

/// Flat types that can be initialized to default state.
///
/// # Safety
///
/// Methods must properly initialize memory.
pub trait FlatDefault: Flat {
    type DefaultEmplacer: Emplacer<Self>;

    /// Initialize uninitialized memory into valid default state.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn default_emplacer() -> Self::DefaultEmplacer;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn default_in_place(bytes: &mut MaybeUninitUnsized<Self>) -> Result<&mut Self, Error> {
        Self::new_in_place(bytes, Self::default_emplacer())
    }
}

impl<T: Flat + Default> FlatDefault for T {
    type DefaultEmplacer = Self;

    fn default_emplacer() -> Self::DefaultEmplacer {
        Self::default()
    }
}
