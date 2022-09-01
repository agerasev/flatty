use crate::{mem::Muu, Error, Flat, FlatCast, Portable};

impl<T: Flat + Sized, const N: usize> FlatCast for [T; N] {
    fn validate(_: &Muu<Self>) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}

unsafe impl<T: Portable + Flat + Sized, const N: usize> Portable for [T; N] {}
