use crate::base::{Flat, FlatInit};
use core::ptr;

/// Primitive flat type.
///
/// # Safety
///
/// Any possible memory representation must be valid.
pub unsafe trait FlatPrim: Flat + Sized + Copy {}

impl<T: FlatPrim> FlatInit for T {
    type Init = Self;
    fn init(mem: &mut [u8], init: Self::Init) -> &mut Self {
        let self_ = unsafe { Self::interpret_mut_unchecked(mem) };
        // Dirty hack because the compiler cannot prove that `Self::Init` is the same as `Self`.
        *self_ = unsafe { ptr::read(&init as *const _ as *const Self) };
        self_
    }

    fn validate(&self) -> bool {
        true
    }
    fn interpret(mem: &[u8]) -> &Self {
        assert!(Self::check_size_and_align(mem));
        unsafe { Self::interpret_unchecked(mem) }
    }
    fn interpret_mut(mem: &mut [u8]) -> &mut Self {
        assert!(Self::check_size_and_align(mem));
        unsafe { Self::interpret_mut_unchecked(mem) }
    }

    unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
        &*(mem.as_ptr() as *const Self)
    }
    unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        &mut *(mem.as_mut_ptr() as *mut Self)
    }
}
unsafe impl<T: FlatPrim> Flat for T {}

unsafe impl FlatPrim for () {}

unsafe impl FlatPrim for bool {}

unsafe impl FlatPrim for u8 {}
unsafe impl FlatPrim for u16 {}
unsafe impl FlatPrim for u32 {}
unsafe impl FlatPrim for u64 {}
unsafe impl FlatPrim for u128 {}

unsafe impl FlatPrim for i8 {}
unsafe impl FlatPrim for i16 {}
unsafe impl FlatPrim for i32 {}
unsafe impl FlatPrim for i64 {}
unsafe impl FlatPrim for i128 {}

unsafe impl FlatPrim for f32 {}
unsafe impl FlatPrim for f64 {}

unsafe impl<T: FlatPrim + Sized, const N: usize> FlatPrim for [T; N] {}
