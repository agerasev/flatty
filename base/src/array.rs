use crate::{mem::MaybeUninitUnsized, Error, Flat, FlatCast};

impl<T: Flat + Sized, const N: usize> FlatCast for [T; N] {
    fn validate(_: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}
