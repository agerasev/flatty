use crate::{mem::MaybeUninitUnsized, Error, Flat, FlatCast};
use core::marker::PhantomData;

impl<T> FlatCast for PhantomData<T> {
    fn validate(_: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T> Flat for PhantomData<T> {}
