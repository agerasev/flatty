use crate::Portable;
use core::marker::PhantomData;
use flatty_base::{
    traits::Flat,
    vec::{FlatVec, Length},
};

unsafe impl<T: Flat> Portable for PhantomData<T> {}

unsafe impl<T: Portable + Flat + Sized, const N: usize> Portable for [T; N] {}

unsafe impl<T, L> Portable for FlatVec<T, L>
where
    T: Portable + Flat + Sized,
    L: Portable + Flat + Length,
{
}
