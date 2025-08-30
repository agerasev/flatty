use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    emplacer::{Emplacer, NeverEmplacer},
    error::{Error, ErrorKind},
    prelude::*,
    utils::{
        ceil_mul, floor_mul,
        iter::{self, prelude::*},
        mem::{set_slice_ptr_len, slice_ptr_len},
        min,
    },
    FlatVec,
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

unsafe impl FlatValidate for UnsizedEnumTag {
    unsafe fn validate_unchecked(__bytes: &[u8]) -> Result<(), Error> {
        u8::validate_unchecked(__bytes)?;
        let tag = u8::from_bytes_unchecked(__bytes);
        if *tag < 3 {
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
    C { offset: &'a u32, bytes: &'a FlatVec<u8, u16> },
}

#[allow(dead_code)]
enum UnsizedEnumMut<'a> {
    A,
    B(&'a mut u8, &'a mut u16),
    C {
        offset: &'a mut u32,
        bytes: &'a mut FlatVec<u8, u16>,
    },
}

#[repr(C)]
struct UnsizedEnumAlignAs(u8, u8, u16, u32, <FlatVec<u8, u16> as FlatUnsized>::AlignAs);

unsafe impl Send for UnsizedEnum {}
unsafe impl Sync for UnsizedEnum {}

#[allow(dead_code)]
enum UnsizedEnumInit<B0, B1, CA, CB> {
    A,
    B(B0, B1),
    C { offset: CA, bytes: CB },
}

#[allow(dead_code)]
struct UnsizedEnumInitA;

#[allow(dead_code)]
struct UnsizedEnumInitB<B0, B1>(pub B0, pub B1);

#[allow(dead_code)]
struct UnsizedEnumInitC<CA, CB> {
    pub offset: CA,
    pub bytes: CB,
}

impl From<UnsizedEnumInitA> for UnsizedEnumInit<NeverEmplacer, NeverEmplacer, NeverEmplacer, NeverEmplacer> {
    fn from(_: UnsizedEnumInitA) -> Self {
        Self::A
    }
}

impl<B0, B1> From<UnsizedEnumInitB<B0, B1>> for UnsizedEnumInit<B0, B1, NeverEmplacer, NeverEmplacer> {
    fn from(this: UnsizedEnumInitB<B0, B1>) -> Self {
        Self::B(this.0, this.1)
    }
}

impl<CA, CB> From<UnsizedEnumInitC<CA, CB>> for UnsizedEnumInit<NeverEmplacer, NeverEmplacer, CA, CB> {
    fn from(this: UnsizedEnumInitC<CA, CB>) -> Self {
        Self::C {
            offset: this.offset,
            bytes: this.bytes,
        }
    }
}

unsafe impl Emplacer<UnsizedEnum> for UnsizedEnumInitA {
    unsafe fn emplace_unchecked(self, __bytes: &mut [u8]) -> Result<&mut UnsizedEnum, Error> {
        UnsizedEnumInit::from(self).emplace_unchecked(__bytes)
    }
}

unsafe impl<B0, B1> Emplacer<UnsizedEnum> for UnsizedEnumInitB<B0, B1>
where
    B0: Emplacer<u8>,
    B1: Emplacer<u16>,
{
    unsafe fn emplace_unchecked(self, __bytes: &mut [u8]) -> Result<&mut UnsizedEnum, Error> {
        UnsizedEnumInit::from(self).emplace_unchecked(__bytes)
    }
}

unsafe impl<CA, CB> Emplacer<UnsizedEnum> for UnsizedEnumInitC<CA, CB>
where
    CA: Emplacer<u32>,
    CB: Emplacer<FlatVec<u8, u16>>,
{
    unsafe fn emplace_unchecked(self, __bytes: &mut [u8]) -> Result<&mut UnsizedEnum, Error> {
        UnsizedEnumInit::from(self).emplace_unchecked(__bytes)
    }
}

impl<B0, B1, CA, CB> UnsizedEnumInit<B0, B1, CA, CB> {
    fn tag(&self) -> UnsizedEnumTag {
        match self {
            Self::A => UnsizedEnumTag::A,
            Self::B(..) => UnsizedEnumTag::B,
            Self::C { .. } => UnsizedEnumTag::C,
        }
    }
}

unsafe impl<B0, B1, CA, CB> Emplacer<UnsizedEnum> for UnsizedEnumInit<B0, B1, CA, CB>
where
    B0: Emplacer<u8>,
    B1: Emplacer<u16>,
    CA: Emplacer<u32>,
    CB: Emplacer<FlatVec<u8, u16>>,
{
    unsafe fn emplace_unchecked(self, __bytes: &mut [u8]) -> Result<&mut UnsizedEnum, Error> {
        self.tag().emplace_unchecked(__bytes)?;
        match self {
            Self::A => (),
            Self::B(b0, b1) => {
                let iter = iter::BytesMutIter::new(
                    unsafe { __bytes.get_unchecked_mut(UnsizedEnum::DATA_OFFSET..) },
                    iter::type_list!(u8, u16),
                )
                .map_err(|e| e.offset(UnsizedEnum::DATA_OFFSET))?;
                let (iter, u0) = iter.next();
                b0.emplace_unchecked(u0)?;
                let u1 = iter.finalize();
                b1.emplace_unchecked(u1)?;
            }
            Self::C { offset, bytes } => {
                let iter = iter::BytesMutIter::new(
                    unsafe { __bytes.get_unchecked_mut(UnsizedEnum::DATA_OFFSET..) },
                    iter::type_list!(u32, FlatVec<u8, u16>),
                )
                .map_err(|e| e.offset(UnsizedEnum::DATA_OFFSET))?;
                let (iter, __u_offset) = iter.next();
                offset.emplace_unchecked(__u_offset)?;
                let __u_bytes = iter.finalize();
                bytes.emplace_unchecked(__u_bytes)?;
            }
        }
        Ok(unsafe { UnsizedEnum::from_mut_bytes_unchecked(__bytes) })
    }
}

impl UnsizedEnum {
    const DATA_OFFSET: usize = ceil_mul(u8::SIZE, Self::ALIGN);
    const LAST_FIELD_OFFSETS: [usize; 3] = [
        0,
        ceil_mul(iter::fold_size!(0; u8), u16::ALIGN),
        ceil_mul(iter::fold_size!(0; u32), FlatVec::<u8, u16>::ALIGN),
    ];
    const DATA_MIN_SIZES: [usize; 3] = [
        0,
        ceil_mul(Self::LAST_FIELD_OFFSETS[1] + u16::MIN_SIZE, Self::ALIGN),
        ceil_mul(Self::LAST_FIELD_OFFSETS[2] + FlatVec::<u8, u16>::MIN_SIZE, Self::ALIGN),
    ];

    pub fn tag(&self) -> UnsizedEnumTag {
        self.tag
    }

    pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
        match self.tag {
            UnsizedEnumTag::A => UnsizedEnumRef::A,
            UnsizedEnumTag::B => {
                let iter = unsafe {
                    iter::RefIter::new_unchecked(iter::UncheckedRefData::new(&self.data), iter::type_list!(u8, u16))
                };
                let (iter, b0) = iter.next();
                let b1 = iter.finalize();
                UnsizedEnumRef::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter = unsafe {
                    iter::RefIter::new_unchecked(
                        iter::UncheckedRefData::new(&self.data),
                        iter::type_list!(u32, FlatVec<u8, u16>),
                    )
                };
                let (iter, offset) = iter.next();
                let bytes = iter.finalize();
                UnsizedEnumRef::C { offset, bytes }
            }
        }
    }
    pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
        match self.tag {
            UnsizedEnumTag::A => UnsizedEnumMut::A,
            UnsizedEnumTag::B => {
                let iter = unsafe {
                    iter::MutIter::new_unchecked(iter::UncheckedMutData::new(&mut self.data), iter::type_list!(u8, u16))
                };
                let (iter, b0) = iter.next();
                let b1 = iter.finalize();
                UnsizedEnumMut::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter = unsafe {
                    iter::MutIter::new_unchecked(
                        iter::UncheckedMutData::new(&mut self.data),
                        iter::type_list!(u32, FlatVec<u8, u16>),
                    )
                };
                let (iter, offset) = iter.next();
                let bytes = iter.finalize();
                UnsizedEnumMut::C { offset, bytes }
            }
        }
    }
}

unsafe impl FlatBase for UnsizedEnum {
    const ALIGN: usize = align_of::<UnsizedEnumAlignAs>();
    const MIN_SIZE: usize =
        Self::DATA_OFFSET + min(Self::DATA_MIN_SIZES[0], min(Self::DATA_MIN_SIZES[1], Self::DATA_MIN_SIZES[2]));

    fn size(&self) -> usize {
        ceil_mul(
            Self::DATA_OFFSET
                + match self.tag {
                    UnsizedEnumTag::A => 0,
                    UnsizedEnumTag::B => unsafe {
                        iter::BytesIter::new_unchecked(&self.data, iter::type_list!(u8, u16)).fold_size(0)
                    },
                    UnsizedEnumTag::C => unsafe {
                        iter::BytesIter::new_unchecked(&self.data, iter::type_list!(u32, FlatVec<u8, u16>)).fold_size(0)
                    },
                },
            Self::ALIGN,
        )
    }
}

unsafe impl FlatUnsized for UnsizedEnum {
    type AlignAs = UnsizedEnumAlignAs;

    unsafe fn ptr_from_bytes(__bytes: *mut [u8]) -> *mut Self {
        set_slice_ptr_len(__bytes, floor_mul(slice_ptr_len(__bytes) - Self::DATA_OFFSET, Self::ALIGN)) as *mut Self
    }
    unsafe fn ptr_to_bytes(this: *mut Self) -> *mut [u8] {
        let __bytes = this as *mut [u8];
        set_slice_ptr_len(__bytes, Self::DATA_OFFSET + slice_ptr_len(__bytes))
    }
}

unsafe impl FlatValidate for UnsizedEnum {
    unsafe fn validate_unchecked(__bytes: &[u8]) -> Result<(), Error> {
        UnsizedEnumTag::validate_unchecked(__bytes)?;
        let tag = UnsizedEnumTag::from_bytes_unchecked(__bytes);
        let __bytes = unsafe { __bytes.get_unchecked(Self::DATA_OFFSET..) };
        if __bytes.len() < Self::DATA_MIN_SIZES[*tag as usize] {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        match tag {
            UnsizedEnumTag::A => Ok(()),
            UnsizedEnumTag::B => unsafe { iter::BytesIter::new_unchecked(__bytes, iter::type_list!(u8, u16)).validate_all() },
            UnsizedEnumTag::C => unsafe {
                iter::BytesIter::new_unchecked(__bytes, iter::type_list!(u32, FlatVec<u8, u16>)).validate_all()
            },
        }
        .map_err(|e| e.offset(Self::DATA_OFFSET))
    }
}

impl FlatDefault for UnsizedEnum {
    type DefaultEmplacer = UnsizedEnumInitA;
    fn default_emplacer() -> Self::DefaultEmplacer {
        UnsizedEnumInitA
    }
}

unsafe impl Flat for UnsizedEnum {}

generate_tests!();
