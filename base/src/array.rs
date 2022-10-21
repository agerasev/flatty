use crate::{mem::MaybeUninitUnsized, Error, Flat, FlatCheck};

impl<T: Flat + Sized, const N: usize> FlatCheck for [T; N] {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error> {
        unsafe { Ok(this.assume_init()) }
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}
