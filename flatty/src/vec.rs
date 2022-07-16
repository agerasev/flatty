use crate::{Flat, FlatExt, FlatLen, FlatSized};
use core::{
    cmp,
    mem::{size_of, MaybeUninit},
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(C)]
pub struct FlatVec<T: Flat + Sized, L: FlatLen = u32> {
    len: L,
    data: [MaybeUninit<T>],
}

impl<T: Flat + Sized, L: FlatLen> FlatVec<T, L> {
    pub fn capacity(&self) -> usize {
        let begin = self.data.as_ptr() as usize;
        let count = (self.data.len() - begin) / size_of::<T>();
        cmp::min(count, L::MAX_USIZE)
    }
    pub fn len(&self) -> usize {
        self.len.into_usize()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

const fn const_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

unsafe impl<T: FlatSized, L: FlatLen> Flat for FlatVec<T, L> {
    const ALIGN: usize = const_max(T::ALIGN, L::ALIGN);
    fn size(&self) -> usize {
        cmp::max(L::SIZE, Self::ALIGN) + T::SIZE * self.len()
    }
}

pub fn make_wide_ptr(slice: &[u8]) -> *const [u8] {
    unsafe { from_raw_parts(slice.as_ptr(), slice.as_ptr() as usize + slice.len()) }
}

pub fn make_wide_ptr_mut(slice: &mut [u8]) -> *mut [u8] {
    unsafe { from_raw_parts_mut(slice.as_mut_ptr(), slice.as_ptr() as usize + slice.len()) }
}

impl<T: FlatSized, L: FlatLen> FlatExt for FlatVec<T, L> {
    fn from_slice(slice: &[u8]) -> &Self {
        assert_eq!(slice.as_ptr().align_offset(Self::ALIGN), 0);
        let mem = make_wide_ptr(slice);
        let ptr = mem as *const [_] as *const Self;
        unsafe { &*ptr }
    }
    fn from_slice_mut(slice: &mut [u8]) -> &mut Self {
        assert_eq!(slice.as_ptr().align_offset(Self::ALIGN), 0);
        let mem = make_wide_ptr_mut(slice);
        let ptr = mem as *mut [_] as *mut Self;
        unsafe { &mut *ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn align() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    #[should_panic]
    fn it_works() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());

        assert_eq!(size_of_val(flat_vec), flat_vec.size());
    }

    #[test]
    fn len_cap() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());

        assert_eq!(flat_vec.len(), 0);
        assert_eq!(flat_vec.capacity(), 3);
    }
}
