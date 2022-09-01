use crate::{mem::Muu, Error, Flat, FlatCast};
use core::marker::PhantomData;

impl<T> FlatCast for PhantomData<T> {
    fn validate(_: &Muu<Self>) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T> Flat for PhantomData<T> {}
