use crate::{
    error::{Error, ErrorKind},
    mem::Muu,
    utils::max,
    Flat, FlatBase, FlatCast, FlatDefault, FlatSized, FlatUnsized,
};
use core::{
    mem::MaybeUninit,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};
use stavec::GenericVec;

pub use stavec::traits::Length;

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
pub type FlatVec<T, L = usize> = GenericVec<T, [MaybeUninit<T>], L>;

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

    fn ptr_from_bytes(bytes: &[u8]) -> *const Self {
        let slice = slice_from_raw_parts(bytes.as_ptr(), Self::ptr_metadata(bytes).unwrap());
        slice as *const [_] as *const Self
    }
    fn ptr_from_mut_bytes(bytes: &mut [u8]) -> *mut Self {
        let slice =
            slice_from_raw_parts_mut(bytes.as_mut_ptr(), Self::ptr_metadata(bytes).unwrap());
        slice as *mut [_] as *mut Self
    }
}

unsafe impl<T, L> FlatUnsized for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    type AlignAs = FlatVecAlignAs<T, L>;

    fn ptr_metadata(bytes: &[u8]) -> Option<usize> {
        Some((bytes.len() - Self::DATA_OFFSET) / T::SIZE)
    }
}

impl<T, L> FlatDefault for FlatVec<T, L>
where
    T: Flat + Sized + Default,
    L: Flat + Length + Default,
{
    fn init_default(this: &mut Muu<Self>) -> Result<(), Error> {
        let len = unsafe { Muu::<L>::from_mut_bytes_unchecked(this.as_mut_bytes()) };
        L::init_default(len)?; // To avoid dereferencing invalid state.
        unsafe { *len.as_mut_ptr() = L::zero() };
        Ok(())
    }
}

impl<T, L> FlatCast for FlatVec<T, L>
where
    T: Flat + Sized,
    L: Flat + Length,
{
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let len = unsafe { &Muu::<L>::from_bytes_unchecked(this.as_bytes()) };
        L::validate(len)?;
        // Now it's safe to dereference `Self`, because data is `[MaybeUninit<T>]`.
        let self_ = unsafe { &*this.as_ptr() };
        if self_.len() > self_.capacity() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        for x in unsafe { self_.data().get_unchecked(..self_.len()) } {
            T::validate(Muu::from_sized(x))?;
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

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::{
        mem::{align_of_val, size_of_val},
        vec,
    };

    #[test]
    fn data_offset() {
        let mut bytes = vec![0u8; 2 + 3 * 4];
        let flat_vec = FlatVec::<i32, u16>::placement_default(bytes.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u16>::ALIGN);
    }

    #[test]
    fn align() {
        let mut bytes = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(bytes.as_mut_slice()).unwrap();

        assert_eq!(align_of_val(flat_vec), FlatVec::<i32, u32>::ALIGN);
    }

    #[test]
    fn len_cap() {
        let mut bytes = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(bytes.as_mut_slice()).unwrap();
        assert_eq!(flat_vec.capacity(), 3);
        assert_eq!(flat_vec.len(), 0);
    }

    #[test]
    fn size() {
        let mut bytes = vec![0u8; 4 + 3 * 4];
        let flat_vec = FlatVec::<i32, u32>::placement_default(bytes.as_mut_slice()).unwrap();
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
        let mut bytes = vec![0u8; 4 * 6];
        let vec = FlatVec::<i32, u32>::placement_default(&mut bytes).unwrap();
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
        let vec_a = FlatVec::<i32, u32>::placement_default(&mut mem_a).unwrap();
        assert_eq!(vec_a.extend_from_slice(&[1, 2, 3, 4]), 4);

        let mut mem_b = vec![0u8; 4 * 5];
        let vec_b = FlatVec::<i32, u32>::placement_default(&mut mem_b).unwrap();
        assert_eq!(vec_b.extend_from_slice(&[1, 2, 3, 4]), 4);

        let mut mem_c = vec![0u8; 4 * 3];
        let vec_c = FlatVec::<i32, u32>::placement_default(&mut mem_c).unwrap();
        assert_eq!(vec_c.extend_from_slice(&[1, 2]), 2);

        assert_eq!(vec_a, vec_b);
        assert_ne!(vec_a, vec_c);
        assert_ne!(vec_b, vec_c);

        vec_b[3] = 5;
        assert_ne!(vec_a, vec_b);
    }
}
