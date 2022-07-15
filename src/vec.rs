use crate::{Flat, FlatExt, FlatSize};
use core::{
    cmp,
    mem::{align_of_val, size_of, MaybeUninit},
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(C)]
pub struct FlatVec<T: Flat + Sized, S: FlatSize = u32> {
    len: S,
    data: [MaybeUninit<T>],
}

impl<T: Flat + Sized, S: FlatSize> FlatVec<T, S> {
    pub fn capacity(&self) -> usize {
        let begin = self.data.as_ptr() as usize;
        let count = (self.data.len() - begin) / size_of::<T>();
        cmp::min(count, S::MAX_USIZE)
    }
    pub fn len(&self) -> usize {
        self.len.into_usize()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn make_wide_ptr(slice: &[u8]) -> *const [u8] {
    unsafe { from_raw_parts(slice.as_ptr(), slice.as_ptr() as usize + slice.len()) }
}

pub fn make_wide_ptr_mut(slice: &mut [u8]) -> *mut [u8] {
    unsafe { from_raw_parts_mut(slice.as_mut_ptr(), slice.as_ptr() as usize + slice.len()) }
}

impl<T: Flat + Sized, S: FlatSize> FlatExt for FlatVec<T, S> {
    fn align_offset(ptr: *const u8) -> usize {
        ptr.align_offset(align_of_val(&ptr))
    }

    fn from_slice(slice: &[u8]) -> &Self {
        assert_eq!(Self::align_offset(slice.as_ptr()), 0);
        let mem = make_wide_ptr(slice);
        let ptr = mem as *const [_] as *const Self;
        unsafe { &*ptr }
    }
    fn from_slice_mut(slice: &mut [u8]) -> &mut Self {
        assert_eq!(Self::align_offset(slice.as_ptr()), 0);
        let mem = make_wide_ptr_mut(slice);
        let ptr = mem as *mut [_] as *mut Self;
        unsafe { &mut *ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());
        assert_eq!(align_of_val(flat_vec), 4);
        assert_eq!(flat_vec.len(), 0);
        assert_eq!(flat_vec.capacity(), 3);
    }
}
