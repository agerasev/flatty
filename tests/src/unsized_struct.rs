use core::mem::{align_of, align_of_val, size_of_val};
use flatty::{
    impl_unsized_uninit_cast,
    iter::{fold_size, prelude::*, type_list, MutIter, RefIter},
    make_flat,
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::ceil_mul,
    Error, FlatVec,
};

//#[make_flat(sized = false)]
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
    const MIN_SIZE: usize = ceil_mul(
        Self::LAST_FIELD_OFFSET + FlatVec::<u64, u32>::MIN_SIZE,
        Self::ALIGN,
    );

    fn size(&self) -> usize {
        ceil_mul(Self::LAST_FIELD_OFFSET + self.c.size(), Self::ALIGN)
    }
}

unsafe impl FlatMaybeUnsized for UnsizedStruct {
    type AlignAs = AlignAs;

    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize {
        FlatVec::<u64, u32>::ptr_metadata(unsafe {
            MaybeUninitUnsized::<FlatVec<u64, u32>>::from_bytes_unchecked(
                &this.as_bytes()[Self::LAST_FIELD_OFFSET..],
            )
        })
    }
    fn bytes_len(this: &Self) -> usize {
        Self::LAST_FIELD_OFFSET + FlatVec::<u64, u32>::bytes_len(&this.c)
    }

    impl_unsized_uninit_cast!();
}

impl FlatCast for UnsizedStruct {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        unsafe { RefIter::new_unchecked(this.as_bytes(), type_list!(u8, u16, FlatVec<u64, u32>)) }
            .validate_all()
    }
}

unsafe impl FlatDefault for UnsizedStruct {
    fn init_default(this: &mut MaybeUninitUnsized<Self>) -> Result<(), Error> {
        unsafe {
            MutIter::new_unchecked(this.as_mut_bytes(), type_list!(u8, u16, FlatVec<u64, u32>))
        }
        .init_default_all()
    }
}

unsafe impl Flat for UnsizedStruct {}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 4 * 8];
    let us = UnsizedStruct::placement_default(mem.as_mut_slice()).unwrap();
    us.a = 200;
    us.b = 40000;
    us.c.extend_from_slice(&[0, 1]);

    assert_eq!(us.size(), 32);
    assert_eq!(us.a, 200);
    assert_eq!(us.b, 40000);
    assert_eq!(us.c.len(), 2);

    for i in 2.. {
        if us.c.push(i).is_err() {
            break;
        }
    }

    assert_eq!(us.size(), 48);
    assert_eq!(us.a, 200);
    assert_eq!(us.b, 40000);
    assert_eq!(us.c.len(), 4);
    assert_eq!(us.c.as_slice(), [0, 1, 2, 3]);
}

#[test]
fn default() {
    let mut mem = vec![0u8; 16 + 4 * 8];
    let us = UnsizedStruct::placement_default(mem.as_mut_slice()).unwrap();

    assert_eq!(us.size(), 16);
    assert_eq!(us.a, 0);
    assert_eq!(us.b, 0);
    assert_eq!(us.c.len(), 0);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 16 + 4 * 8];
    let us = UnsizedStruct::placement_default(mem.as_mut_slice()).unwrap();
    us.a = 0;
    us.b = 0;
    for i in 0.. {
        if us.c.push(i).is_err() {
            break;
        }
    }

    assert_eq!(align_of_val(us), <UnsizedStruct as FlatBase>::ALIGN);
    assert_eq!(size_of_val(us), us.size());
    assert_eq!(us.size(), mem.len());
}

#[test]
fn eq() {
    let mut mem_ab = vec![0u8; 16 + 4 * 8];
    let mut mem_c = vec![0u8; 16 + 3 * 8];
    {
        let us = UnsizedStruct::placement_default(&mut mem_ab).unwrap();
        us.a = 1;
        us.b = 2;
        us.c.extend_from_slice(&[3, 4, 5, 6]);
    }
    let us_a = UnsizedStruct::from_bytes(&mem_ab).unwrap();
    let us_b = UnsizedStruct::from_bytes(&mem_ab).unwrap();
    let us_c = UnsizedStruct::placement_default(&mut mem_c).unwrap();
    us_c.a = 1;
    us_c.b = 2;
    us_c.c.extend_from_slice(&[3, 4, 5]);

    assert_eq!(us_a, us_b);
    assert_ne!(us_a, us_c);
    assert_ne!(us_b, us_c);
}
