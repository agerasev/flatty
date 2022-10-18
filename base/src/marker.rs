use crate::{mem::MaybeUninitUnsized, Error, Flat, FlatCheck};
use core::marker::PhantomData;

impl<T> FlatCheck for PhantomData<T> {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error> {
        unsafe { Ok(this.assume_init()) }
    }
}

unsafe impl<T> Flat for PhantomData<T> {}
