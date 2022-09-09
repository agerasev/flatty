use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    impl_unsized_uninit_cast,
    iter::{fold_size, prelude::*, type_list, MutIter, RefIter},
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::{ceil_mul, floor_mul, min},
    Error, ErrorKind, FlatVec,
};

#[repr(C)]
struct UnsizedEnum {
    tag: UnsizedEnumTag,
    _align: [UnsizedEnumAlignAs; 0],
    data: [u8],
}

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
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
                kind: ErrorKind::InvalidEnumTag,
                pos: 0,
            })
        }
    }
}

unsafe impl Flat for UnsizedEnumTag {}

#[allow(dead_code)]
enum UnsizedEnumRef<'a> {
    A,
    B(&'a u8, &'a u16),
    C { a: &'a u8, b: &'a FlatVec<u8, u16> },
}

#[allow(dead_code)]
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
    pub fn set_default(&mut self, tag: UnsizedEnumTag) -> Result<(), Error> {
        self.tag = tag;
        unsafe { Self::init_default_data_by_tag(tag, &mut self.data) }
            .map_err(|e| e.offset(Self::DATA_OFFSET))
    }
    unsafe fn init_default_data_by_tag(tag: UnsizedEnumTag, bytes: &mut [u8]) -> Result<(), Error> {
        if bytes.len() < Self::DATA_MIN_SIZES[tag as u8 as usize] {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: 0,
            });
        }
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

generate_tests!();
