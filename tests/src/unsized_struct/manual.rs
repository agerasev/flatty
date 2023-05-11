use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    impl_unsized_uninit_cast,
    mem::Unvalidated,
    prelude::*,
    utils::{
        ceil_mul,
        iter::{self, prelude::*},
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

impl<A, B, C> Emplacer<UnsizedStruct> for UnsizedStructInit<A, B, C>
where
    A: Emplacer<u8>,
    B: Emplacer<u16>,
    C: Emplacer<FlatVec<u64, u32>>,
{
    fn emplace(self, uninit: &mut Unvalidated<UnsizedStruct>) -> Result<&mut UnsizedStruct, Error> {
        let iter =
            unsafe { iter::MutIter::new_unchecked(uninit.as_mut_bytes(), iter::type_list!(u8, u16, FlatVec<u64, u32>)) };
        let (iter, u_a) = iter.next();
        self.a.emplace(u_a)?;
        let (iter, u_b) = iter.next();
        self.b.emplace(u_b)?;
        let u_c = iter.finalize();
        self.c.emplace(u_c)?;
        Ok(unsafe { uninit.assume_init_mut() })
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

    fn ptr_metadata(this: &Unvalidated<Self>) -> usize {
        FlatVec::<u64, u32>::ptr_metadata(unsafe {
            Unvalidated::<FlatVec<u64, u32>>::from_bytes_unchecked(&this.as_bytes()[Self::LAST_FIELD_OFFSET..])
        })
    }
    fn bytes_len(this: &Self) -> usize {
        Self::LAST_FIELD_OFFSET + FlatVec::<u64, u32>::bytes_len(&this.c)
    }

    impl_unsized_uninit_cast!();
}

impl FlatValidate for UnsizedStruct {
    fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error> {
        unsafe { iter::RefIter::new_unchecked(this.as_bytes(), iter::type_list!(u8, u16, FlatVec<u64, u32>)) }
            .validate_all()?;
        Ok(unsafe { this.assume_init() })
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
