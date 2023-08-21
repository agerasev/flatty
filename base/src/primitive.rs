use crate::{
    error::Error,
    traits::{Flat, FlatSized, FlatValidate},
};
use core::marker::PhantomData;

/// Macro for implementing [`Flat`] for primitive types.
///
/// # Safety
///
/// `$ty` must be [`Sized`]` + `[`Copy`].
///
/// Any possible memory state of the variable of the type must be valid.
macro_rules! impl_flat_prim {
    ($ty:ty) => {
        unsafe impl FlatValidate for $ty {
            unsafe fn validate_unchecked(_: &[u8]) -> Result<(), Error> {
                Ok(())
            }
        }
        unsafe impl Flat for $ty {}
    };
}

impl_flat_prim!(());

impl_flat_prim!(u8);
impl_flat_prim!(u16);
impl_flat_prim!(u32);
impl_flat_prim!(u64);
impl_flat_prim!(u128);
impl_flat_prim!(usize);

impl_flat_prim!(i8);
impl_flat_prim!(i16);
impl_flat_prim!(i32);
impl_flat_prim!(i64);
impl_flat_prim!(i128);
impl_flat_prim!(isize);

impl_flat_prim!(f32);
impl_flat_prim!(f64);

unsafe impl<T: Flat> FlatValidate for PhantomData<T> {
    unsafe fn validate_unchecked(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
unsafe impl<T: Flat> Flat for PhantomData<T> {}

unsafe impl<T: Flat, const N: usize> FlatValidate for [T; N] {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        for i in 0..N {
            T::validate_unchecked(bytes.get_unchecked((i * T::SIZE)..).get_unchecked(..T::SIZE))?;
        }
        Ok(())
    }
}
unsafe impl<T: Flat, const N: usize> Flat for [T; N] {}
