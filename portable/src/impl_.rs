use crate::Portable;
use core::marker::PhantomData;
use flatty_containers::vec::{FlatVec, Length};

unsafe impl<T: Portable> Portable for PhantomData<T> {}

unsafe impl<T: Portable + Sized, const N: usize> Portable for [T; N] {}

unsafe impl<T, L> Portable for FlatVec<T, L>
where
    T: Portable + Sized,
    L: Portable + Length,
{
}
