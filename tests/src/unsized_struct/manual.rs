use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    prelude::*,
    utils::{
        ceil_mul,
        iter::{self, prelude::*},
        mem::{cast_wide_ptr_with_offset, offset_slice_ptr_start},
    },
    Emplacer, Error, FlatVec,
};

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64, u32>,
}

#[repr(C)]
struct AlignAs(u8, u16, <FlatVec<u64, u32> as FlatUnsized>::AlignAs);

struct UnsizedStructInit<A, B, C> {
    a: A,
    b: B,
    c: C,
}

unsafe impl<A, B, C> Emplacer<UnsizedStruct> for UnsizedStructInit<A, B, C>
where
    A: Emplacer<u8>,
    B: Emplacer<u16>,
    C: Emplacer<FlatVec<u64, u32>>,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<(), Error> {
        let iter = iter::BytesMutIter::new_unchecked(bytes, iter::type_list!(u8, u16, FlatVec<u64, u32>));
        let (iter, u_a) = iter.next();
        self.a.emplace_unchecked(u_a)?;
        let (iter, u_b) = iter.next();
        self.b.emplace_unchecked(u_b)?;
        let u_c = iter.finalize();
        self.c.emplace_unchecked(u_c)?;
        Ok(())
    }
}

impl UnsizedStruct {
    const LAST_FIELD_OFFSET: usize = ceil_mul(iter::fold_size!(0; u8, u16), FlatVec::<u64, u32>::ALIGN);
}

unsafe impl FlatBase for UnsizedStruct {
    const ALIGN: usize = align_of::<AlignAs>();
    const MIN_SIZE: usize = ceil_mul(iter::fold_min_size!(0; u8, u16, FlatVec<u64, u32>), Self::ALIGN);

    fn size(&self) -> usize {
        ceil_mul(Self::LAST_FIELD_OFFSET + self.c.size(), Self::ALIGN)
    }
}

unsafe impl FlatUnsized for UnsizedStruct {
    type AlignAs = AlignAs;

    unsafe fn ptr_from_bytes(bytes: *mut [u8]) -> *mut Self {
        cast_wide_ptr_with_offset!(
            Self,
            FlatVec::<u64, u32>::ptr_from_bytes(offset_slice_ptr_start(bytes, Self::LAST_FIELD_OFFSET as isize)),
            -(Self::LAST_FIELD_OFFSET as isize),
        )
    }
    unsafe fn ptr_to_bytes(this: *mut Self) -> *mut [u8] {
        offset_slice_ptr_start(
            FlatVec::<u64, u32>::ptr_to_bytes(cast_wide_ptr_with_offset!(
                FlatVec::<u64, u32>,
                this,
                Self::LAST_FIELD_OFFSET as isize
            )),
            -(Self::LAST_FIELD_OFFSET as isize),
        )
    }
}

unsafe impl FlatValidate for UnsizedStruct {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        unsafe { iter::BytesIter::new_unchecked(bytes, iter::type_list!(u8, u16, FlatVec<u64, u32>)) }.validate_all()
    }
}

impl FlatDefault for UnsizedStruct {
    type DefaultEmplacer = UnsizedStructInit<u8, u16, <FlatVec<u64, u32> as FlatDefault>::DefaultEmplacer>;
    fn default_emplacer() -> Self::DefaultEmplacer {
        UnsizedStructInit {
            a: u8::default(),
            b: u16::default(),
            c: <FlatVec<u64, u32> as FlatDefault>::default_emplacer(),
        }
    }
}

unsafe impl Flat for UnsizedStruct {}

generate_tests!();
