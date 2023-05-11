use crate::{mem::Unvalidated, Error, Flat, FlatValidate};

impl<T: Flat + Sized, const N: usize> FlatValidate for [T; N] {
    fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error> {
        unsafe { Ok(this.assume_init()) }
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}
