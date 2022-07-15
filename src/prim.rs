use crate::{Flat, FlatExt};
use core::mem::align_of;

unsafe impl Flat for bool {}

unsafe impl Flat for u8 {}
unsafe impl Flat for u16 {}
unsafe impl Flat for u32 {}
unsafe impl Flat for u64 {}
unsafe impl Flat for u128 {}

unsafe impl Flat for i8 {}
unsafe impl Flat for i16 {}
unsafe impl Flat for i32 {}
unsafe impl Flat for i64 {}
unsafe impl Flat for i128 {}

unsafe impl Flat for f32 {}
unsafe impl Flat for f64 {}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}

impl<T: Flat + Sized> FlatExt for T {
    fn align_offset(ptr: *const u8) -> usize {
        ptr.align_offset(align_of::<Self>())
    }

    fn from_slice(mem: &[u8]) -> &Self {
        assert_eq!(Self::align_offset(mem.as_ptr()), 0);
        let ptr = mem.as_ptr() as *const Self;
        unsafe { &*ptr }
    }
    fn from_slice_mut(mem: &mut [u8]) -> &mut Self {
        assert_eq!(Self::align_offset(mem.as_ptr()), 0);
        let ptr = mem.as_mut_ptr() as *mut Self;
        unsafe { &mut *ptr }
    }
}
