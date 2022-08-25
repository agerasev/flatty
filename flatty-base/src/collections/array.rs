use crate::{Error, Flat, FlatInit, FlatSized, Portable};
use core::mem::size_of;

impl<T: Flat + Sized, const N: usize> FlatInit for [T; N] {
    type Dyn = [T::Dyn; N];
    fn size_of(_: &Self::Dyn) -> usize {
        T::SIZE * N
    }

    unsafe fn placement_new_unchecked<'a, 'b>(
        mem: &'a mut [u8],
        init: &'b Self::Dyn,
    ) -> &'a mut Self {
        for (i, ii) in init.iter().enumerate() {
            T::placement_new_unchecked(&mut mem[(i * size_of::<T>())..][..size_of::<T>()], ii);
        }
        Self::reinterpret_mut_unchecked(mem)
    }

    fn pre_validate(mem: &[u8]) -> Result<(), Error> {
        for i in 0..N {
            T::pre_validate(&mem[i * size_of::<T>()..][..size_of::<T>()])?;
        }
        Ok(())
    }
    fn post_validate(&self) -> Result<(), Error> {
        for item in self {
            item.post_validate()?;
        }
        Ok(())
    }

    unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
        &*(mem.as_ptr() as *const Self)
    }
    unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        &mut *(mem.as_mut_ptr() as *mut Self)
    }
}

unsafe impl<T: Flat + Sized, const N: usize> Flat for [T; N] {}

unsafe impl<T: Portable + Flat + Sized, const N: usize> Portable for [T; N] {}
