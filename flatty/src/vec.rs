use crate::{
    base::{Flat, FlatBase, FlatInit},
    len::FlatLen,
    sized::FlatSized,
    util::{const_max, slice_assume_init_mut, slice_assume_init_ref},
};
use core::{
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(C)]
pub struct FlatVec<T: Flat + Sized, L: FlatLen = u32> {
    len: L,
    data: [MaybeUninit<T>],
}

impl<T: Flat + Sized, L: FlatLen> FlatVec<T, L> {
    const DATA_OFFSET: usize = const_max(L::SIZE, T::ALIGN);

    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    pub fn len(&self) -> usize {
        self.len.into_usize()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub fn push(&mut self, x: T) -> Result<(), T> {
        if !self.is_full() {
            self.data[self.len()] = MaybeUninit::new(x);
            self.len += L::from_usize(1).unwrap();
            Ok(())
        } else {
            Err(x)
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            self.len -= L::from_usize(1).unwrap();
            Some(unsafe { self.data[self.len()].assume_init_read() })
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { slice_assume_init_ref(&self.data[..len]) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe { slice_assume_init_mut(&mut self.data[..len]) }
    }
}

#[repr(C, u8)]
pub enum FlatVecAlignAs<T: Flat + Sized, L: FlatLen> {
    Len(L),
    Item(T),
}

impl<T: Flat + Sized, L: FlatLen> FlatBase for FlatVec<T, L> {
    type AlignAs = FlatVecAlignAs<T, L>;

    const MIN_SIZE: usize = Self::DATA_OFFSET;
    fn size(&self) -> usize {
        Self::DATA_OFFSET + T::SIZE * self.len()
    }
}

pub enum FlatVecInit {
    Empty,
}
impl Default for FlatVecInit {
    fn default() -> Self {
        FlatVecInit::Empty
    }
}

impl<T: Flat + Sized, L: FlatLen> FlatInit for FlatVec<T, L> {
    type Init = FlatVecInit;
    fn init(mem: &mut [u8], init: Self::Init) -> &mut Self {
        let self_ = unsafe { Self::interpret_mut_unchecked(mem) };
        match init {
            FlatVecInit::Empty => {
                self_.len = L::from_usize(0).unwrap();
            }
        }
        self_
    }

    fn validate(&self) -> bool {
        if self.len() > self.capacity() {
            return false;
        }
        for x in self.as_slice() {
            if !x.validate() {
                return false;
            }
        }
        true
    }
    fn interpret(mem: &[u8]) -> &Self {
        Self::check_size_and_align(mem);
        let self_ = unsafe { Self::interpret_unchecked(mem) };
        assert!(self_.validate());
        self_
    }
    fn interpret_mut(mem: &mut [u8]) -> &mut Self {
        Self::check_size_and_align(mem);
        let self_ = unsafe { Self::interpret_mut_unchecked(mem) };
        assert!(self_.validate());
        self_
    }

    unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
        let slice = from_raw_parts(mem.as_ptr(), (mem.len() - Self::DATA_OFFSET) / T::SIZE);
        &*(slice as *const [_] as *const Self)
    }
    unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        let slice = from_raw_parts_mut(mem.as_mut_ptr(), (mem.len() - Self::DATA_OFFSET) / T::SIZE);
        &mut *(slice as *mut [_] as *mut Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn data_offset() {
        let mut mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<i32, u16>::init_default(mem.as_mut_slice());

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn align() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::init_default(mem.as_mut_slice());

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::init_default(mem.as_mut_slice());
        assert_eq!(flat_vec.capacity(), 3);

        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::init_default(mem.as_mut_slice());
        assert_eq!(FlatVec::<i32>::DATA_OFFSET, flat_vec.size());

        for i in 0.. {
            if flat_vec.push(i).is_err() {
                break;
            }
        }
        assert_eq!(flat_vec.len(), 3);
        assert_eq!(size_of_val(flat_vec), flat_vec.size());
    }
}
