use crate::{
    utils::{max, slice_assume_init_mut, slice_assume_init_ref},
    Error, Flat, FlatBase, FlatInit, FlatLen, FlatSized, FlatUnsized,
};
use core::{
    cmp::{Eq, PartialEq},
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
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
    type Dyn = Vec<T::Dyn>;
    fn size_of(value: &Self::Dyn) -> usize {
        T::SIZE * value.len()
    }

    unsafe fn placement_new_unchecked<'a, 'b>(
        mem: &'a mut [u8],
        init: &'b Self::Dyn,
    ) -> &'a mut Self {
        let self_ = Self::reinterpret_mut_unchecked(mem);
        self_.len = L::from_usize(init.len()).unwrap();
        for (src, dst) in init.iter().zip(self_.data.iter_mut()) {
            T::placement_new_unchecked(
                from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, T::SIZE),
                src,
            );
        }
        self_
    }
    fn placement_new<'a, 'b>(
        mem: &'a mut [u8],
        init: &'b Self::Dyn,
    ) -> Result<&'a mut Self, Error> {
        Self::check_size_and_align(mem)?;
        if mem.len() - Self::DATA_OFFSET < T::SIZE * init.len() {
            return Err(Error::InsufficientSize);
        }
        Ok(unsafe { Self::placement_new_unchecked(mem, init) })
    }

    fn pre_validate(mem: &[u8]) -> Result<(), Error> {
        let len = unsafe { L::reinterpret_unchecked(mem) }.into_usize();
        let data = &mem[Self::DATA_OFFSET..];
        if len > data.len() / T::SIZE {
            return Err(Error::InsufficientSize);
        }
        for i in 0..len {
            T::pre_validate(unsafe { from_raw_parts(data.as_ptr().add(i * T::SIZE), T::SIZE) })?;
        }
        Ok(())
    }
    fn post_validate(&self) -> Result<(), Error> {
        if self.len() > self.capacity() {
            unreachable!();
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

impl<T: Flat + Sized, L: FlatLen> Deref for FlatVec<T, L> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Flat + Sized, L: FlatLen> DerefMut for FlatVec<T, L> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Flat + Sized + Eq, L: FlatLen> PartialEq for FlatVec<T, L> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x == y)
    }
}

impl<T: Flat + Sized + Eq, L: FlatLen> Eq for FlatVec<T, L> {}

impl<T: Flat + Sized + Debug, L: FlatLen> Debug for FlatVec<T, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

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

    #[test]
    fn eq() {
        let mut mem_a = vec![0u8; 4 * 5];
        let mut mem_b = vec![0u8; 4 * 5];
        let mut mem_c = vec![0u8; 4 * 3];
        let vec_a = FlatVec::<i32>::placement_new(&mut mem_a, &vec![1, 2, 3, 4]).unwrap();
        let vec_b = FlatVec::<i32>::placement_new(&mut mem_b, &vec![1, 2, 3, 4]).unwrap();
        let vec_c = FlatVec::<i32>::placement_new(&mut mem_c, &vec![1, 2]).unwrap();

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }
}
