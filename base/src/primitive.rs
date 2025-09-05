use crate::{
    error::Error,
    traits::{Flat, FlatSized, FlatValidate},
    utils::ceil_mul,
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

unsafe impl<T: ?Sized> FlatValidate for PhantomData<T> {
    unsafe fn validate_unchecked(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
unsafe impl<T: ?Sized> Flat for PhantomData<T> {}

unsafe impl<T: Flat, const N: usize> FlatValidate for [T; N] {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        for i in 0..N {
            T::validate_unchecked(bytes.get_unchecked((i * T::SIZE)..).get_unchecked(..T::SIZE))?;
        }
        Ok(())
    }
}
unsafe impl<T: Flat, const N: usize> Flat for [T; N] {}

macro_rules! impl_flat_tuple {
    ($( $param:ident ),* $(,)?) => {
        unsafe impl<$( $param ),*> FlatValidate for ( $( $param, )* )
            where $( $param: Flat ),*
        {
            #[allow(unused_assignments)]
            unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
                let mut offset = 0;
                $(
                    offset = ceil_mul(offset, $param::ALIGN);
                    <$param as FlatValidate>::validate_unchecked(
                        bytes.get_unchecked(offset..).get_unchecked(..$param::SIZE)
                    )?;
                    offset += $param::SIZE;
                )*
                Ok(())
            }
        }
        unsafe impl<$( $param ),*> Flat for ( $( $param, )* )
            where $( $param: Flat ),*
        {}
    };
}

impl_flat_tuple!(A);
impl_flat_tuple!(A, B);
impl_flat_tuple!(A, B, C);
impl_flat_tuple!(A, B, C, D);
impl_flat_tuple!(A, B, C, D, E);
impl_flat_tuple!(A, B, C, D, E, F);
impl_flat_tuple!(A, B, C, D, E, F, G);
impl_flat_tuple!(A, B, C, D, E, F, G, H);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_flat_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
