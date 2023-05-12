use crate::{
    emplacer::Emplacer,
    error::{Error, ErrorKind},
    traits::{Flat, FlatBase, FlatDefault, FlatSized, FlatUnsized, FlatValidate},
    utils::{floor_mul, max},
};
use core::{
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};
use stavec::GenericVec;

pub use stavec::traits::{Length, Slot};

#[repr(transparent)]
pub struct MaybeInvalid<T: FlatSized>(MaybeUninit<T>);
impl<T: FlatSized> MaybeInvalid<T> {
    fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}
unsafe impl<T: FlatSized> Slot for MaybeInvalid<T> {
    type Item = T;

    fn new(item: Self::Item) -> Self {
        Self(MaybeUninit::new(item))
    }
    unsafe fn assume_init(self) -> Self::Item {
        self.0.assume_init()
    }
    unsafe fn assume_init_read(&self) -> Self::Item {
        self.0.assume_init_read()
    }
}

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
pub type FlatVec<T, L = usize> = GenericVec<[MaybeInvalid<T>], L>;

trait DataOffset<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    const DATA_OFFSET: usize = max(L::SIZE, T::ALIGN);
}

impl<T, L> DataOffset<T, L> for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
}

/// Sized type that has same alignment as [`FlatVec<T, L>`](`FlatVec`).
#[repr(C)]
pub struct FlatVecAlignAs<T, L>(T, L)
where
    T: Flat + Sized,
    L: Flat + Length;

unsafe impl<T, L> FlatBase for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    const ALIGN: usize = max(L::ALIGN, T::ALIGN);
    const MIN_SIZE: usize = Self::DATA_OFFSET;

    fn size(&self) -> usize {
        Self::DATA_OFFSET + T::SIZE * self.len()
    }
}

unsafe impl<T, L> FlatUnsized for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    type AlignAs = FlatVecAlignAs<T, L>;

    fn ptr_from_bytes(bytes: &[u8]) -> *const Self {
        let meta = floor_mul(bytes.len() - Self::DATA_OFFSET, Self::ALIGN) / T::SIZE;
        ptr::slice_from_raw_parts(bytes.as_ptr(), meta) as *const Self
    }
    unsafe fn ptr_to_bytes<'a>(this: *const Self) -> &'a [u8] {
        let meta = unsafe { NonNull::new_unchecked(this as *mut [T]) }.len();
        let len = Self::DATA_OFFSET + meta * T::SIZE;
        slice::from_raw_parts(this as *const u8, len)
    }
}

pub struct Empty;
pub struct FromArray<T, const N: usize>(pub [T; N]);
pub struct FromIterator<T, I: Iterator<Item = T>>(pub I);

unsafe impl<T, L> Emplacer<FlatVec<T, L>> for Empty
where
    T: Flat + Sized,
    L: Flat + Length,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<(), Error> {
        unsafe { (bytes.as_mut_ptr() as *mut L).write(L::zero()) };
        // Now it's safe to assume that `Self` is initialized, because vector data is `[MaybeInvalid<T>]`.
        Ok(())
    }
}

unsafe impl<T, L, const N: usize> Emplacer<FlatVec<T, L>> for FromArray<T, N>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<(), Error> {
        unsafe { <Empty as Emplacer<FlatVec<T, L>>>::emplace_unchecked(Empty, bytes) }?;
        let vec = unsafe { FlatVec::<T, L>::from_mut_bytes_unchecked(bytes) };
        if vec.capacity() < N {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: 0,
            });
        }
        assert_eq!(vec.extend_from_iter(self.0.into_iter()), N);
        Ok(())
    }
}

unsafe impl<T, L, I: Iterator<Item = T>> Emplacer<FlatVec<T, L>> for FromIterator<T, I>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<(), Error> {
        unsafe { <Empty as Emplacer<FlatVec<T, L>>>::emplace_unchecked(Empty, bytes) }?;
        let vec = unsafe { FlatVec::<T, L>::from_mut_bytes_unchecked(bytes) };
        for x in self.0 {
            if vec.push(x).is_err() {
                return Err(Error {
                    kind: ErrorKind::InsufficientSize,
                    pos: 0,
                });
            }
        }
        Ok(())
    }
}

impl<T, L> FlatDefault for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    type DefaultEmplacer = Empty;

    fn default_emplacer() -> Empty {
        Empty
    }
}

unsafe impl<T, L> FlatValidate for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        unsafe { L::validate_unchecked(bytes) }?;
        // Now it's safe to assume that `Self` is initialized, because vector data is `[MaybeInvalid<T>]`.
        let this = unsafe { Self::from_bytes_unchecked(bytes) };
        if this.len() > this.capacity() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        for x in unsafe { this.data().get_unchecked(..this.len()) } {
            unsafe { T::validate_unchecked(T::ptr_to_bytes(x.as_ptr())) }?;
        }
        Ok(())
    }
}

unsafe impl<T, L> Flat for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
}

/// Creates [`FlatVec`] emplacer from given array.
#[macro_export]
macro_rules! flat_vec {
    () => {
        $crate::vec::FromArray([])
    };
    ($elem:expr; $n:expr) => {
        $crate::vec::FromArray([$elem; $n])
    };
    ($($x:expr),+ $(,)?) => {
        $crate::vec::FromArray([$($x),+])
    };
}
pub use flat_vec;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::utils::alloc::AlignedBytes;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn data_offset() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u16>::default_in_place(&mut bytes).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u16>::ALIGN);
    }

    #[test]
    fn align() {
        let mut bytes = AlignedBytes::new(4 + 3 * 2, 4);
        let flat_vec = FlatVec::<i16, u32>::default_in_place(&mut bytes).unwrap();

        assert_eq!(align_of_val(flat_vec), 4);
        assert_eq!(flat_vec.capacity(), 2);
        assert_eq!(size_of_val(flat_vec), 8);
    }

    #[test]
    fn len_cap() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u32>::default_in_place(&mut bytes).unwrap();
        assert_eq!(flat_vec.capacity(), 3);
        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u32>::default_in_place(&mut bytes).unwrap();
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
        let mut bytes = AlignedBytes::new(4 * 6, 4);
        let vec = FlatVec::<i32, u32>::default_in_place(&mut bytes).unwrap();
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
        let mut mem_a = AlignedBytes::new(4 * 5, 4);
        let vec_a = FlatVec::<i32, u32>::new_in_place(&mut mem_a, flat_vec![1, 2, 3, 4]).unwrap();

        let mut mem_b = AlignedBytes::new(4 * 5, 4);
        let vec_b = FlatVec::<i32, u32>::new_in_place(&mut mem_b, flat_vec![1, 2, 3, 4]).unwrap();

        let mut mem_c = AlignedBytes::new(4 * 3, 4);
        let vec_c = FlatVec::<i32, u32>::new_in_place(&mut mem_c, flat_vec![1, 2]).unwrap();

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }
}
