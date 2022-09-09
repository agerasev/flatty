use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    impl_unsized_uninit_cast,
    iter::{fold_min_size, fold_size, prelude::*, type_list, MutIter, RefIter},
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::ceil_mul,
    Error, FlatVec,
};

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64, u32>,
}

#[repr(C)]
struct AlignAs(u8, u16, <FlatVec<u64, u32> as FlatMaybeUnsized>::AlignAs);

impl UnsizedStruct {
    const LAST_FIELD_OFFSET: usize = ceil_mul(fold_size!(0; u8, u16), FlatVec::<u64, u32>::ALIGN);
}

unsafe impl FlatBase for UnsizedStruct {
    const ALIGN: usize = align_of::<AlignAs>();
    const MIN_SIZE: usize = ceil_mul(fold_min_size!(0; u8, u16, FlatVec<u64, u32>), Self::ALIGN);

    fn size(&self) -> usize {
        ceil_mul(Self::LAST_FIELD_OFFSET + self.c.size(), Self::ALIGN)
    }
}

unsafe impl FlatMaybeUnsized for UnsizedStruct {
    type AlignAs = AlignAs;

    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize {
        FlatVec::<u64, u32>::ptr_metadata(unsafe {
            MaybeUninitUnsized::<FlatVec<u64, u32>>::from_bytes_unchecked(&this.as_bytes()[Self::LAST_FIELD_OFFSET..])
        })
    }
    fn bytes_len(this: &Self) -> usize {
        Self::LAST_FIELD_OFFSET + FlatVec::<u64, u32>::bytes_len(&this.c)
    }

    impl_unsized_uninit_cast!();
}

impl FlatCast for UnsizedStruct {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        unsafe { RefIter::new_unchecked(this.as_bytes(), type_list!(u8, u16, FlatVec<u64, u32>)) }.validate_all()
    }
}

unsafe impl FlatDefault for UnsizedStruct {
    fn init_default(this: &mut MaybeUninitUnsized<Self>) -> Result<(), Error> {
        unsafe { MutIter::new_unchecked(this.as_mut_bytes(), type_list!(u8, u16, FlatVec<u64, u32>)) }.init_default_all()
    }
}

unsafe impl Flat for UnsizedStruct {}

generate_tests!();
