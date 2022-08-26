use crate::{
    utils::{max, slice_assume_init_mut, slice_assume_init_ref},
    Error, Flat, FlatBase, FlatInit, FlatSized, FlatUnsized, Portable,
};
use core::{
    cmp::{Eq, PartialEq},
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice::{from_raw_parts, from_raw_parts_mut},
};
use num_traits::{FromPrimitive, ToPrimitive, Unsigned};

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
#[repr(C)]
pub struct FlatVec<T, L = usize>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    len: L,
    data: [MaybeUninit<T>],
}

impl<T, L> FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    const DATA_OFFSET: usize = max(L::SIZE, T::ALIGN);

    /// Maximum number of items could be stored in this vector.
    ///
    /// The capacity is determined by its reference metadata.
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    /// Number of items stored in the vactor.
    pub fn len(&self) -> usize {
        self.len.to_usize().unwrap()
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
            self.len = self.len + L::one();
            Ok(())
        } else {
            Err(x)
        }
    }
    /// Take and return an item from the end of the vector.
    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            self.len = self.len - L::one();
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
pub struct FlatVecAlignAs<T, L>(L, T)
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive;

impl<T, L> FlatBase for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    const ALIGN: usize = max(L::ALIGN, T::ALIGN);

    const MIN_SIZE: usize = Self::DATA_OFFSET;
    fn size(&self) -> usize {
        Self::DATA_OFFSET + T::SIZE * self.len()
    }
}

impl<T, L> FlatUnsized for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    type AlignAs = FlatVecAlignAs<T, L>;

    fn ptr_metadata(mem: &[u8]) -> usize {
        (mem.len() - Self::DATA_OFFSET) / T::SIZE
    }
}

#[cfg(feature = "std")]
pub type FlatVecDyn<T> = std::vec::Vec<T>;
#[cfg(not(feature = "std"))]
pub type FlatVecDyn<T> = [T; 0];

impl<T, L> FlatInit for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    type Dyn = FlatVecDyn<T::Dyn>;
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
        let len = unsafe { L::reinterpret_unchecked(mem) }.to_usize().unwrap();
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

unsafe impl<T, L> Flat for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
}

unsafe impl<T, L> Portable for FlatVec<T, L>
where
    T: Portable + Flat + Sized,
    L: Portable + Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
}

impl<T, L> Deref for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L> DerefMut for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, L> PartialEq for FlatVec<T, L>
where
    T: PartialEq + Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x == y)
    }
}

impl<T, L> Eq for FlatVec<T, L>
where
    T: Eq + Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
}

impl<T, L> Debug for FlatVec<T, L>
where
    T: Debug + Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::portable::le;
    use std::{
        mem::{align_of_val, size_of_val},
        vec,
    };

    #[test]
    fn data_offset() {
        let mut mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<i32, u16>::placement_default(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u16>::ALIGN);
    }

    #[test]
    fn align() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(mem.as_mut_slice()).unwrap();
        assert_eq!(flat_vec.capacity(), 3);

        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(mem.as_mut_slice()).unwrap();
        assert_eq!(FlatVec::<i32, u32>::DATA_OFFSET, flat_vec.size());

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
        let vec_a = FlatVec::<i32, u32>::placement_new(&mut mem_a, &vec![1, 2, 3, 4]).unwrap();
        let vec_b = FlatVec::<i32, u32>::placement_new(&mut mem_b, &vec![1, 2, 3, 4]).unwrap();
        let vec_c = FlatVec::<i32, u32>::placement_new(&mut mem_c, &vec![1, 2]).unwrap();

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }

    #[test]
    fn primitive() {
        let mut mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<le::I32, le::U16>::placement_default(mem.as_mut_slice()).unwrap();

        flat_vec.push(le::I32::from(0)).unwrap();
        flat_vec.push(le::I32::from(1)).unwrap();
        flat_vec.push(le::I32::from(2)).unwrap();
        assert!(flat_vec.push(le::I32::from(3)).is_err());

        assert_eq!(FlatVec::<le::I32, le::U16>::ALIGN, 1);
        assert_eq!(align_of_val(flat_vec), 1);
    }
}
