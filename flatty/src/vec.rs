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

const fn const_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
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
}

#[repr(C, u8)]
pub enum FlatVecAlignAs<T: Flat + Sized, L: FlatLen> {
    Len(L),
    Item(T),
}

unsafe impl<T: FlatSized, L: FlatLen> Flat for FlatVec<T, L> {
    type AlignAs = FlatVecAlignAs<T, L>;

    fn size(&self) -> usize {
        Self::DATA_OFFSET + T::SIZE * self.len()
    }
}

impl<T: FlatSized, L: FlatLen> FlatExt for FlatVec<T, L> {
    fn from_slice(slice: &[u8]) -> &Self {
        assert!(slice.len() >= Self::DATA_OFFSET);
        assert!(slice.as_ptr().align_offset(Self::ALIGN) == 0);

        let mem =
            unsafe { from_raw_parts(slice.as_ptr(), (slice.len() - Self::DATA_OFFSET) / T::SIZE) };
        let ptr = mem as *const [_] as *const Self;
        unsafe { &*ptr }
    }
    fn from_slice_mut(slice: &mut [u8]) -> &mut Self {
        assert!(slice.len() >= Self::DATA_OFFSET);
        assert!(slice.as_ptr().align_offset(Self::ALIGN) == 0);

        let mem = unsafe {
            from_raw_parts_mut(
                slice.as_mut_ptr(),
                (slice.len() - Self::DATA_OFFSET) / T::SIZE,
            )
        };
        let ptr = mem as *mut [_] as *mut Self;
        unsafe { &mut *ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn data_offset() {
        let mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<i32, u16>::from_slice(mem.as_slice());

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn align() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice(mem.as_slice());
        assert_eq!(flat_vec.capacity(), 3);

        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::from_slice_mut(mem.as_mut_slice());
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
