use core::mem::{align_of, align_of_val, size_of_val};
use flatty::{
    impl_unsized_uninit_cast,
    iter::{fold_size, prelude::*, type_list, MutIter, RefIter},
    make_flat,
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::{ceil_mul, floor_mul, max, min},
    Error, ErrorKind, FlatVec,
};

/*
#[make_flat(sized = false, enum_type = "u8")]
enum UnsizedEnum {
    #[default]
    A,
    B(u8, u16),
    C { a: u8, b: FlatVec<u8, u16> },
}
*/

#[repr(C)]
struct UnsizedEnum {
    tag: UnsizedEnumTag,
    _align: [UnsizedEnumAlignAs; 0],
    data: [u8],
}

#[repr(u8)]
#[derive(Clone, Copy, Default)]
enum UnsizedEnumTag {
    #[default]
    A = 0,
    B,
    C,
}

impl FlatCast for UnsizedEnumTag {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        let tag = unsafe { MaybeUninitUnsized::<u8>::from_bytes_unchecked(this.as_bytes()) };
        u8::validate(tag)?;
        if *unsafe { tag.assume_init_ref() } < 3 {
            Ok(())
        } else {
            Err(Error {
                kind: ErrorKind::InvalidEnumState,
                pos: 0,
            })
        }
    }
}

unsafe impl Flat for UnsizedEnumTag {}

enum UnsizedEnumRef<'a> {
    A,
    B(&'a u8, &'a u16),
    C { a: &'a u8, b: &'a FlatVec<u8, u16> },
}

enum UnsizedEnumMut<'a> {
    A,
    B(&'a mut u8, &'a mut u16),
    C {
        a: &'a mut u8,
        b: &'a mut FlatVec<u8, u16>,
    },
}

#[repr(C)]
struct UnsizedEnumAlignAs(
    u8,
    u8,
    u16,
    u8,
    <FlatVec<u8, u16> as FlatMaybeUnsized>::AlignAs,
);

impl UnsizedEnum {
    const DATA_OFFSET: usize = ceil_mul(u8::SIZE, Self::ALIGN);
    const LAST_FIELD_OFFSETS: [usize; 3] = [
        0,
        ceil_mul(fold_size!(0; u8), u16::ALIGN),
        ceil_mul(fold_size!(0; u8), FlatVec::<u8, u16>::ALIGN),
    ];
    const DATA_MIN_SIZES: [usize; 3] = [
        0,
        ceil_mul(Self::LAST_FIELD_OFFSETS[1] + u16::MIN_SIZE, Self::ALIGN),
        ceil_mul(
            Self::LAST_FIELD_OFFSETS[2] + FlatVec::<u8, u16>::MIN_SIZE,
            Self::ALIGN,
        ),
    ];

    pub fn tag(&self) -> UnsizedEnumTag {
        self.tag
    }
    pub fn set_tag(&mut self, tag: UnsizedEnumTag) -> Result<(), Error> {
        unsafe { Self::init_default_data_by_tag(tag, &mut self.data) }
    }
    unsafe fn init_default_data_by_tag(tag: UnsizedEnumTag, bytes: &mut [u8]) -> Result<(), Error> {
        match tag {
            UnsizedEnumTag::A => Ok(()),
            UnsizedEnumTag::B => {
                MutIter::new_unchecked(bytes, type_list!(u8, u16)).init_default_all()
            }
            UnsizedEnumTag::C => {
                MutIter::new_unchecked(bytes, type_list!(u8, FlatVec<u8, u16>)).init_default_all()
            }
        }
        .map_err(|e| e.offset(Self::DATA_OFFSET))
    }

    pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
        match self.tag {
            UnsizedEnumTag::A => UnsizedEnumRef::A,
            UnsizedEnumTag::B => {
                let iter = unsafe { RefIter::new_unchecked(&self.data, type_list!(u8, u16)) };
                let (iter, value) = iter.next();
                let b0 = unsafe { value.assume_init_ref() };
                let value = iter.finalize();
                let b1 = unsafe { value.assume_init_ref() };
                UnsizedEnumRef::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter =
                    unsafe { RefIter::new_unchecked(&self.data, type_list!(u8, FlatVec<u8, u16>)) };
                let (iter, value) = iter.next();
                let a = unsafe { value.assume_init_ref() };
                let value = iter.finalize();
                let b = unsafe { value.assume_init_ref() };
                UnsizedEnumRef::C { a, b }
            }
        }
    }
    pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
        match self.tag {
            UnsizedEnumTag::A => UnsizedEnumMut::A,
            UnsizedEnumTag::B => {
                let iter = unsafe { MutIter::new_unchecked(&mut self.data, type_list!(u8, u16)) };
                let (iter, value) = iter.next();
                let b0 = unsafe { value.assume_init_mut() };
                let value = iter.finalize();
                let b1 = unsafe { value.assume_init_mut() };
                UnsizedEnumMut::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter = unsafe {
                    MutIter::new_unchecked(&mut self.data, type_list!(u8, FlatVec<u8, u16>))
                };
                let (iter, value) = iter.next();
                let a = unsafe { value.assume_init_mut() };
                let value = iter.finalize();
                let b = unsafe { value.assume_init_mut() };
                UnsizedEnumMut::C { a, b }
            }
        }
    }
}

unsafe impl FlatBase for UnsizedEnum {
    const ALIGN: usize = align_of::<UnsizedEnumAlignAs>();
    const MIN_SIZE: usize = Self::DATA_OFFSET
        + min(
            Self::DATA_MIN_SIZES[0],
            min(Self::DATA_MIN_SIZES[1], Self::DATA_MIN_SIZES[2]),
        );

    fn size(&self) -> usize {
        ceil_mul(
            Self::DATA_OFFSET
                + match self.tag {
                    UnsizedEnumTag::A => 0,
                    UnsizedEnumTag::B => unsafe {
                        RefIter::new_unchecked(&self.data, type_list!(u8, u16)).fold_size(0)
                    },
                    UnsizedEnumTag::C => unsafe {
                        RefIter::new_unchecked(&self.data, type_list!(u8, FlatVec<u8, u16>))
                            .fold_size(0)
                    },
                },
            Self::ALIGN,
        )
    }
}

unsafe impl FlatMaybeUnsized for UnsizedEnum {
    type AlignAs = UnsizedEnumAlignAs;

    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize {
        floor_mul(this.as_bytes().len() - Self::DATA_OFFSET, Self::ALIGN)
    }
    fn bytes_len(this: &Self) -> usize {
        Self::DATA_OFFSET + this.data.len()
    }

    impl_unsized_uninit_cast!();
}

impl FlatCast for UnsizedEnum {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        let bytes = this.as_bytes();
        let tag = unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_bytes_unchecked(bytes) };
        UnsizedEnumTag::validate(tag)?;
        let bytes = unsafe { bytes.get_unchecked(Self::DATA_OFFSET..) };
        match unsafe { tag.assume_init_ref() } {
            UnsizedEnumTag::A => Ok(()),
            UnsizedEnumTag::B => unsafe {
                RefIter::new_unchecked(bytes, type_list!(u8, u16)).validate_all()
            },
            UnsizedEnumTag::C => unsafe {
                RefIter::new_unchecked(bytes, type_list!(u8, FlatVec<u8, u16>)).validate_all()
            },
        }
        .map_err(|e| e.offset(Self::DATA_OFFSET))
    }
}

unsafe impl FlatDefault for UnsizedEnum {
    fn init_default(this: &mut MaybeUninitUnsized<Self>) -> Result<(), Error> {
        let bytes = this.as_mut_bytes();
        let tag = unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_mut_bytes_unchecked(bytes) };
        UnsizedEnumTag::init_default(tag)?;
        unsafe {
            Self::init_default_data_by_tag(
                *tag.assume_init_ref(),
                bytes.get_unchecked_mut(Self::DATA_OFFSET..),
            )
        }
    }
}

unsafe impl Flat for UnsizedEnum {}

#[test]
fn init_a() {
    let mut mem = vec![0u8; 2];
    let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    assert_eq!(ue.size(), 2);

    match ue.as_ref() {
        UnsizedEnumRef::A => (),
        _ => panic!(),
    }

    assert_eq!(mem[0], 0);
}

#[test]
fn init_b() {
    let mut mem = vec![0u8; 6];
    let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    ue.set_tag(UnsizedEnumTag::B).unwrap();
    if let UnsizedEnumMut::B(b0, b1) = ue.as_mut() {
        *b0 = 0xab;
        *b1 = 0xcdef;
    } else {
        unreachable!();
    }
    assert_eq!(ue.size(), 6);

    match ue.as_ref() {
        UnsizedEnumRef::B(x, y) => {
            assert_eq!(*x, 0xab);
            assert_eq!(*y, 0xcdef);
        }
        _ => panic!(),
    }

    assert_eq!(mem[0], 1);
    assert_eq!(mem[2], 0xab);
    assert_eq!(&mem[4..], [0xef, 0xcd]);
}

#[test]
fn init_c() {
    let mut mem = vec![0u8; 12];
    let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    ue.set_tag(UnsizedEnumTag::C).unwrap();
    if let UnsizedEnumMut::C { a, b } = ue.as_mut() {
        *a = 0xab;
        b.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]);
    } else {
        unreachable!();
    }
    assert_eq!(ue.size(), 10);

    match ue.as_mut() {
        UnsizedEnumMut::C { a, b } => {
            assert_eq!(*a, 0xab);
            assert_eq!(b.len(), 4);
            assert_eq!(b.capacity(), 6);
            b.push(0x9a).unwrap();
            b.push(0xbc).unwrap();
            assert!(b.push(0xde).is_err());
        }
        _ => panic!(),
    }
    assert_eq!(ue.size(), 12);

    assert_eq!(mem[0], 2);
    assert_eq!(mem[2], 0xab);
    assert_eq!(&mem[4..6], [6, 0]);
    assert_eq!(&mem[6..], [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
}

#[test]
fn init_err() {
    let mut mem = vec![0u8; 1];
    let res = UnsizedEnum::placement_default(mem.as_mut_slice());
    assert_eq!(res.err().unwrap().kind, ErrorKind::InsufficientSize);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 6 + 8 * 2 + 1];
    let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    ue.set_tag(UnsizedEnumTag::C).unwrap();
    if let UnsizedEnumMut::C { a, .. } = ue.as_mut() {
        *a = 0xab;
    } else {
        unreachable!();
    }

    if let UnsizedEnumMut::C { b, .. } = ue.as_mut() {
        for i in 0.. {
            if b.push(i).is_err() {
                break;
            }
        }
    } else {
        panic!();
    }

    assert_eq!(UnsizedEnum::DATA_OFFSET, 2);
    assert_eq!(align_of_val(ue), <UnsizedEnum as FlatBase>::ALIGN);
    assert_eq!(size_of_val(ue), ue.size());
    assert_eq!(ue.size(), mem.len() - 1);
}
