use crate::{
    utils::{max, slice_assume_init_mut, slice_assume_init_ref},
    Error, Flat, FlatBase, FlatInit, FlatLen, FlatSized, FlatUnsized,
};
use core::{
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
#[repr(C)]
pub struct FlatVec<T: Flat + Sized, L: FlatLen = u32> {
    len: L,
    data: [MaybeUninit<T>],
}

impl<T: Flat + Sized, L: FlatLen> FlatVec<T, L> {
    const DATA_OFFSET: usize = max(L::SIZE, T::ALIGN);

    /// Maximum number of items could be stored in this vector.
    ///
    /// The capacity is determined by its reference metadata.
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    /// Number of items stored in the vactor.
    pub fn len(&self) -> usize {
        self.len.into_usize()
    }
    /// Whether the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Whether the vector is full.
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Put a new item to the end of the vector.
    pub fn push(&mut self, x: T) -> Result<(), T> {
        if !self.is_full() {
            self.data[self.len()] = MaybeUninit::new(x);
            self.len += L::from_usize(1).unwrap();
            Ok(())
        } else {
            Err(x)
        }
    }
    /// Take and return an item from the end of the vector.
    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            self.len -= L::from_usize(1).unwrap();
            Some(unsafe { self.data[self.len()].assume_init_read() })
        } else {
            None
        }
    }

    /// Return a slice of stored items.
    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { slice_assume_init_ref(&self.data[..len]) }
    }
    /// Return a mutable slice of stored items.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe { slice_assume_init_mut(&mut self.data[..len]) }
    }
}

/// Sized type that has same alignment as [`FlatVec<T, L>`](`FlatVec`).
#[repr(C)]
pub struct FlatVecAlignAs<T: Flat + Sized, L: FlatLen>(L, T);

impl<T: Flat + Sized, L: FlatLen> FlatBase for FlatVec<T, L> {
    const ALIGN: usize = max(L::ALIGN, T::ALIGN);

    const MIN_SIZE: usize = Self::DATA_OFFSET;
    fn size(&self) -> usize {
        Self::DATA_OFFSET + T::SIZE * self.len()
    }
}

impl<T: Flat + Sized, L: FlatLen> FlatUnsized for FlatVec<T, L> {
    type AlignAs = FlatVecAlignAs<T, L>;

    fn ptr_metadata(mem: &[u8]) -> usize {
        (mem.len() - Self::DATA_OFFSET) / T::SIZE
    }
}

impl<T: Flat + Sized, L: FlatLen> FlatInit for FlatVec<T, L> {
    type Init = Vec<T>;
    unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
        let self_ = Self::reinterpret_mut_unchecked(mem);
        for x in init.into_iter() {
            assert!(self_.push(x).is_ok());
        }
        self_
    }
    fn placement_new(mem: &mut [u8], init: Self::Init) -> Result<&mut Self, Error> {
        Self::check_size_and_align(mem)?;
        let self_ = unsafe { Self::placement_new_unchecked(mem, Vec::new()) };
        for x in init.into_iter() {
            if self_.push(x).is_err() {
                return Err(Error::InsufficientSize);
            }
        }
        Ok(self_)
    }

    fn pre_validate(_mem: &[u8]) -> Result<(), Error> {
        Ok(())
    }
    fn post_validate(&self) -> Result<(), Error> {
        if self.len() > self.capacity() {
            return Err(Error::InsufficientSize);
        }
        for i in 0..self.len() {
            T::pre_validate(unsafe {
                from_raw_parts((self.data.as_ptr() as *const u8).add(i * T::SIZE), T::SIZE)
            })?;
        }
        for x in self.as_slice() {
            x.post_validate()?;
        }
        Ok(())
    }

    unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
        let slice = from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
        &*(slice as *const [_] as *const Self)
    }
    unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        let slice = from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
        &mut *(slice as *mut [_] as *mut Self)
    }
}

unsafe impl<T: Flat + Sized, L: FlatLen> Flat for FlatVec<T, L> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn data_offset() {
        let mut mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<i32, u16>::placement_default(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn align() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::placement_default(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::placement_default(mem.as_mut_slice()).unwrap();
        assert_eq!(flat_vec.capacity(), 3);

        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32>::placement_default(mem.as_mut_slice()).unwrap();
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
