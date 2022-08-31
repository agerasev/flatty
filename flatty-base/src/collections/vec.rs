use crate::{
    utils::max, Error, ErrorKind, Flat, FlatBase, FlatCast, FlatDefault, FlatSized, FlatUnsized,
    Portable,
};
use core::{
    cmp::{Eq, PartialEq},
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};
use num_traits::{FromPrimitive, ToPrimitive, Unsigned};

/// Assume that slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const [MaybeUninit<T>] as *const [T])
}

/// Assume that mutable slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

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
    /// Number of remaining free places in the vector.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.len.to_usize().unwrap()
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

    /// Clones and appends elements in a slice to this vector until slice ends or vector capacity reached.
    ///
    /// Returns a number of elements being appended.
    pub fn extend_from_slice(&mut self, other: &[T]) -> usize
    where
        T: Clone,
    {
        let mut counter = 0;
        for x in other {
            match self.push(x.clone()) {
                Ok(()) => (),
                Err(_) => break,
            }
            counter += 1;
        }
        counter
    }
}

/// Sized type that has same alignment as [`FlatVec<T, L>`](`FlatVec`).
#[repr(C)]
pub struct FlatVecAlignAs<T, L>(T, L)
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

    fn ptr_from_bytes(mem: &[u8]) -> *const Self {
        let slice = slice_from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
        slice as *const [_] as *const Self
    }
    fn ptr_from_mut_bytes(mem: &mut [u8]) -> *mut Self {
        let slice = slice_from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
        slice as *mut [_] as *mut Self
    }
}

unsafe impl<T, L> FlatUnsized for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    type AlignAs = FlatVecAlignAs<T, L>;

    fn ptr_metadata(mem: &[u8]) -> usize {
        (mem.len() - Self::DATA_OFFSET) / T::SIZE
    }
}

impl<T, L> FlatDefault for FlatVec<T, L>
where
    T: Flat + Sized + Default,
    L: Flat + Sized + Copy + Default + Unsigned + ToPrimitive + FromPrimitive,
{
    unsafe fn default_contents(bytes: &mut [u8]) -> Result<(), Error> {
        L::default_contents(bytes)?; // To avoid dereferencing invalid state.
        *L::ptr_from_mut_bytes(bytes) = L::zero();
        Ok(())
    }
}

impl<T, L> FlatCast for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Sized + Copy + Unsigned + ToPrimitive + FromPrimitive,
{
    unsafe fn validate_contents(bytes: &[u8]) -> Result<(), Error> {
        L::validate_contents(bytes)?;
        // Now it's safe to dereference `Self`, because data is `[MaybeUninit<T>]`.
        let self_ = &*Self::ptr_from_bytes(bytes);
        if self_.len() > self_.capacity() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        for x in self_.data.get_unchecked(..self_.len()) {
            let x_bytes = &*slice_from_raw_parts(x.as_ptr() as *const u8, T::SIZE);
            T::validate_contents(x_bytes)?;
        }
        Ok(())
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
        let flat_vec = FlatVec::<i32, u16>::default_in_bytes(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u16>::ALIGN);
    }

    #[test]
    fn align() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::default_in_bytes(mem.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::default_in_bytes(mem.as_mut_slice()).unwrap();
        assert_eq!(flat_vec.capacity(), 3);
        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut mem = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::default_in_bytes(mem.as_mut_slice()).unwrap();
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
    fn extend_from_slice() {
        let mut mem = vec![0u8; 4 * 6];
        let vec = FlatVec::<i32, u32>::default_in_bytes(&mut mem).unwrap();
        assert_eq!(vec.capacity(), 5);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.remaining(), 5);

        assert_eq!(vec.extend_from_slice(&[1, 2, 3]), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.remaining(), 2);
        assert_eq!(vec.as_slice(), &[1, 2, 3][..]);

        assert_eq!(vec.extend_from_slice(&[4, 5, 6]), 2);
        assert_eq!(vec.len(), 5);
        assert_eq!(vec.remaining(), 0);
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5][..]);
    }

    #[test]
    fn eq() {
        let mut mem_a = vec![0u8; 4 * 5];
        let vec_a = FlatVec::<i32, u32>::default_in_bytes(&mut mem_a).unwrap();
        assert_eq!(vec_a.extend_from_slice(&[1, 2, 3, 4]), 4);

        let mut mem_b = vec![0u8; 4 * 5];
        let vec_b = FlatVec::<i32, u32>::default_in_bytes(&mut mem_b).unwrap();
        assert_eq!(vec_b.extend_from_slice(&[1, 2, 3, 4]), 4);

        let mut mem_c = vec![0u8; 4 * 3];
        let vec_c = FlatVec::<i32, u32>::default_in_bytes(&mut mem_c).unwrap();
        assert_eq!(vec_c.extend_from_slice(&[1, 2]), 2);

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }

    #[test]
    fn primitive() {
        let mut mem = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<le::I32, le::U16>::default_in_bytes(mem.as_mut_slice()).unwrap();

        flat_vec.push(le::I32::from(0)).unwrap();
        flat_vec.push(le::I32::from(1)).unwrap();
        flat_vec.push(le::I32::from(2)).unwrap();
        assert!(flat_vec.push(le::I32::from(3)).is_err());

        assert_eq!(FlatVec::<le::I32, le::U16>::ALIGN, 1);
        assert_eq!(align_of_val(flat_vec), 1);
    }
}
