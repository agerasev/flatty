use crate::{Error, Flat, FlatCast, Portable};

impl<T: Flat + Sized, const N: usize> FlatCast for [T; N] {
    unsafe fn validate_contents(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}

unsafe impl<T: Portable + Flat + Sized, const N: usize> Portable for [T; N] {}
