use crate::{mem::Unvalidated, Error, Flat, FlatValidate};
use core::marker::PhantomData;

impl<T> FlatValidate for PhantomData<T> {
    fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error> {
        unsafe { Ok(this.assume_init()) }
    }
}

unsafe impl<T> Flat for PhantomData<T> {}
