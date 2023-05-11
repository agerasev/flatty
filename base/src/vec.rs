use crate::{
    emplacer::Emplacer,
    error::{Error, ErrorKind},
    impl_unsized_uninit_cast,
    mem::Unvalidated,
    utils::{floor_mul, max},
    Flat, FlatBase, FlatDefault, FlatSized, FlatUnsized, FlatValidate,
};
use core::{
    mem::{self, MaybeUninit},
    ptr::NonNull,
};
use stavec::GenericVec;

pub use stavec::traits::{Length, Slot};

#[repr(transparent)]
pub struct Zeroed<T: FlatSized>(MaybeUninit<T>);
impl<T: FlatSized> Zeroed<T> {
    fn as_unval(&self) -> &Unvalidated<T> {
        unsafe { T::ptr_as_uninit(self.0.as_ptr()) }
    }
}
unsafe impl<T: FlatSized> Slot for Zeroed<T> {
    type Item = T;

    fn empty() -> Self {
        unsafe { Self(mem::zeroed()) }
    }
    fn occupied(item: Self::Item) -> Self {
        Self(MaybeUninit::new(item))
    }
    unsafe fn assume_occupied(self) -> Self::Item {
        self.0.assume_init()
    }
}

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
pub type FlatVec<T, L = usize> = GenericVec<[Zeroed<T>], L>;

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

    fn ptr_metadata(this: &Unvalidated<Self>) -> usize {
        floor_mul(this.as_bytes().len() - Self::DATA_OFFSET, Self::ALIGN) / T::SIZE
    }

    fn bytes_len(this: *const Self) -> usize {
        Self::DATA_OFFSET + unsafe { NonNull::new_unchecked(this as *mut [T]) }.len() * T::SIZE
    }

    impl_unsized_uninit_cast!();
}

pub struct Empty;
pub struct FromArray<T, const N: usize>(pub [T; N]);
pub struct FromIterator<T, I: Iterator<Item = T>>(pub I);

impl<T, L> Emplacer<FlatVec<T, L>> for Empty
where
    T: Flat + Sized,
    L: Flat + Length,
{
    fn emplace(self, uninit: &mut Unvalidated<FlatVec<T, L>>) -> Result<&mut FlatVec<T, L>, Error> {
        unsafe {
            (uninit.as_mut_ptr() as *mut L).write(L::zero());
            // Now it's safe to assume that `Self` is initialized, because vector data is `[MaybeUninit<T>]`.
            Ok(uninit.assume_init_mut())
        }
    }
}

impl<T, L, const N: usize> Emplacer<FlatVec<T, L>> for FromArray<T, N>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    fn emplace(self, uninit: &mut Unvalidated<FlatVec<T, L>>) -> Result<&mut FlatVec<T, L>, Error> {
        let vec = Empty.emplace(uninit).unwrap();
        if vec.capacity() < N {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: 0,
            });
        }
        assert_eq!(vec.extend_from_iter(self.0.into_iter()), N);
        Ok(vec)
    }
}

impl<T, L, I: Iterator<Item = T>> Emplacer<FlatVec<T, L>> for FromIterator<T, I>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    fn emplace(self, uninit: &mut Unvalidated<FlatVec<T, L>>) -> Result<&mut FlatVec<T, L>, Error> {
        let vec = Empty.emplace(uninit).unwrap();
        for x in self.0 {
            if vec.push(x).is_err() {
                return Err(Error {
                    kind: ErrorKind::InsufficientSize,
                    pos: 0,
                });
            }
        }
        Ok(vec)
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

impl<T, L> FlatValidate for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error> {
        let len = unsafe { &Unvalidated::<L>::from_bytes_unchecked(this.as_bytes()) };
        L::validate(len)?;
        // Now it's safe to assume that `Self` is initialized, because vector data is `[MaybeUninit<T>]`.
        let self_ = unsafe { this.assume_init() };
        if self_.len() > self_.capacity() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        for x in unsafe { self_.data().get_unchecked(..self_.len()) } {
            T::validate(x.as_unval())?;
        }
        Ok(self_)
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

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::utils::alloc::AlignedBytes;
    use std::mem::{align_of_val, size_of_val};

    #[test]
    fn data_offset() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u16>::from_mut_bytes(&mut bytes)
            .unwrap()
            .default_in_place()
            .unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u16>::ALIGN);
    }

    #[test]
    fn align() {
        let mut bytes = AlignedBytes::new(4 + 3 * 2, 4);
        let flat_vec = FlatVec::<i16, u32>::from_mut_bytes(&mut bytes)
            .unwrap()
            .default_in_place()
            .unwrap();

        assert_eq!(align_of_val(flat_vec), 4);
        assert_eq!(flat_vec.capacity(), 2);
        assert_eq!(size_of_val(flat_vec), 8);
    }

    #[test]
    fn len_cap() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u32>::from_mut_bytes(&mut bytes)
            .unwrap()
            .default_in_place()
            .unwrap();
        assert_eq!(flat_vec.capacity(), 3);
        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut bytes = AlignedBytes::new(4 + 3 * 4, 4);
        let flat_vec = FlatVec::<i32, u32>::from_mut_bytes(&mut bytes)
            .unwrap()
            .default_in_place()
            .unwrap();
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
        let vec = FlatVec::<i32, u32>::from_mut_bytes(&mut bytes)
            .unwrap()
            .default_in_place()
            .unwrap();
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
        let vec_a = FlatVec::<i32, u32>::from_mut_bytes(&mut mem_a)
            .unwrap()
            .new_in_place(flat_vec![1, 2, 3, 4])
            .unwrap();

        let mut mem_b = AlignedBytes::new(4 * 5, 4);
        let vec_b = FlatVec::<i32, u32>::from_mut_bytes(&mut mem_b)
            .unwrap()
            .new_in_place(flat_vec![1, 2, 3, 4])
            .unwrap();

        let mut mem_c = AlignedBytes::new(4 * 3, 4);
        let vec_c = FlatVec::<i32, u32>::from_mut_bytes(&mut mem_c)
            .unwrap()
            .new_in_place(flat_vec![1, 2])
            .unwrap();

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }
}
