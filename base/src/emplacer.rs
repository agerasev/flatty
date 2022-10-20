use crate::{error::Error, mem::MaybeUninitUnsized, FlatUnsized};

/// In-place initializer of flat type.
///
/// # Safety
///
/// [`Self::init`] must be valid.
pub trait Emplacer<T: FlatUnsized + ?Sized>: Sized {
    /// Apply initializer for uninitizalized memory.
    ///
    /// *In case of success must return reference to the same memory as `uninit`.*
    fn emplace(self, uninit: &mut MaybeUninitUnsized<T>) -> Result<&mut T, Error>;
}

pub enum NeverEmplacer {}

impl<T: FlatUnsized + ?Sized> Emplacer<T> for NeverEmplacer {
    fn emplace(self, _: &mut MaybeUninitUnsized<T>) -> Result<&mut T, Error> {
        unreachable!()
    }
}
