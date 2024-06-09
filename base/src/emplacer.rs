use crate::{
    error::Error,
    traits::{FlatSized, FlatUnsized},
    utils::mem::check_align_and_min_size,
};

/// In-place initializer of flat type.
pub unsafe trait Emplacer<T: FlatUnsized + ?Sized>: Sized {
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut T, Error>;

    /// Apply initializer for uninitialized memory.
    fn emplace(self, bytes: &mut [u8]) -> Result<&mut T, Error> {
        check_align_and_min_size::<T>(bytes)?;
        unsafe { self.emplace_unchecked(bytes) }
    }
}

/// Emplacer that cannot be instantiated and used as a placeholder for unused parameters.
pub enum NeverEmplacer {}

unsafe impl<T: FlatUnsized + ?Sized> Emplacer<T> for NeverEmplacer {
    unsafe fn emplace_unchecked(self, _: &mut [u8]) -> Result<&mut T, Error> {
        unreachable!()
    }
}

unsafe impl<T: FlatSized> Emplacer<T> for T {
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut T, Error> {
        let ptr = Self::ptr_from_bytes(bytes);
        unsafe { ptr.write(self) };
        Ok(unsafe { &mut *ptr })
    }
}
