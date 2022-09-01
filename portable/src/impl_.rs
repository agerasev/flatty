use crate::Portable;
use base::{Flat, FlatVec};
use core::marker::PhantomData;
use num_traits::{FromPrimitive, ToPrimitive, Unsigned};

unsafe impl<T> Portable for PhantomData<T> {}

unsafe impl<T: Portable + Flat + Sized, const N: usize> Portable for [T; N] {}

unsafe impl<T, L> Portable for FlatVec<T, L>
where
    T: Portable + Flat + Sized,
    L: Portable + Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
}
