use crate::{Error, Flat, FlatInit};
use core::marker::PhantomData;

impl<T> FlatInit for PhantomData<T> {
    type Init = Self;
    unsafe fn placement_new_unchecked(mem: &mut [u8], _: Self::Init) -> &mut Self {
        &mut *(mem as *mut _ as *mut PhantomData<T>)
    }

    fn pre_validate(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
    fn post_validate(&self) -> Result<(), Error> {
        Ok(())
    }

    unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
        &*(mem as *const _ as *const PhantomData<T>)
    }
    unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        &mut *(mem as *mut _ as *mut PhantomData<T>)
    }
}

unsafe impl<T> Flat for PhantomData<T> {}
