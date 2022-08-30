use crate::{Error, Flat, FlatCast, Portable};
use core::marker::PhantomData;

impl<T> FlatCast for PhantomData<T> {
    unsafe fn validate(_ptr: *const Self) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<T> Flat for PhantomData<T> {}

unsafe impl<T> Portable for PhantomData<T> {}
