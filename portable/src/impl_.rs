use crate::Portable;
use core::marker::PhantomData;
use flatty_containers::{
    flex::FlexVec,
    string::FlatString,
    vec::{FlatVec, Length},
};

unsafe impl<T: Portable> Portable for PhantomData<T> {}

unsafe impl<T: Portable + Sized, const N: usize> Portable for [T; N] {}

unsafe impl<T, L> Portable for FlatVec<T, L>
where
    T: Portable + Sized,
    L: Portable + Length,
{
}

unsafe impl<L> Portable for FlatString<L> where L: Portable + Length {}

unsafe impl<T, L> Portable for FlexVec<T, L>
where
    T: Portable + ?Sized,
    L: Portable + Length,
{
}

macro_rules! impl_flat_tuple {
    ($( $param:ident ),* $(,)?) => {
        unsafe impl<$( $param ),*> Portable for ( $( $param, )* )
            where $( $param: Portable ),*
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
