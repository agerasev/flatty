use super::tests::generate_tests;
use core::mem::align_of;
use flatty::{
    impl_unsized_uninit_cast,
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::{
        ceil_mul, floor_mul,
        iter::{self, prelude::*},
        min,
    },
    Emplacer, Error, ErrorKind, FlatVec,
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

impl FlatCheck for UnsizedEnumTag {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error> {
        let tag = unsafe { MaybeUninitUnsized::<u8>::from_bytes_unchecked(this.as_bytes()) };
        u8::validate(tag)?;
        if *unsafe { tag.assume_init() } < 3 {
            Ok(unsafe { this.assume_init() })
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
    C { a: &'a mut u8, b: &'a mut FlatVec<u8, u16> },
}

#[repr(C)]
struct UnsizedEnumAlignAs(u8, u8, u16, u8, <FlatVec<u8, u16> as FlatUnsized>::AlignAs);

struct UnsizedEnumInitA;

#[allow(dead_code)]
struct UnsizedEnumInitB<__Last: Emplacer<u16>>(u8, __Last);

#[allow(dead_code)]
struct UnsizedEnumInitC<__Last: Emplacer<FlatVec<u8, u16>>> {
    a: u8,
    b: __Last,
}

impl Emplacer<UnsizedEnum> for UnsizedEnumInitA {
    fn emplace(self, uninit: &mut MaybeUninitUnsized<UnsizedEnum>) -> Result<&mut UnsizedEnum, Error> {
        let bytes = uninit.as_mut_bytes();
        unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_mut_bytes_unchecked(bytes) }
            .as_mut_sized()
            .write(UnsizedEnumTag::A);
        Ok(unsafe { uninit.assume_init_mut() })
    }
}

#[allow(dead_code)]
impl<__Last: Emplacer<u16>> Emplacer<UnsizedEnum> for UnsizedEnumInitB<__Last> {
    fn emplace(self, uninit: &mut MaybeUninitUnsized<UnsizedEnum>) -> Result<&mut UnsizedEnum, Error> {
        let bytes = uninit.as_mut_bytes();
        unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_mut_bytes_unchecked(bytes) }
            .as_mut_sized()
            .write(UnsizedEnumTag::B);
        let iter = iter::MutIter::new(
            unsafe { bytes.get_unchecked_mut(UnsizedEnum::DATA_OFFSET..) },
            iter::type_list!(u8, u16),
        )
        .map_err(|e| e.offset(UnsizedEnum::DATA_OFFSET))?;
        let (iter, u_0) = iter.next();
        self.0.emplace(u_0)?;
        let u_1 = iter.finalize();
        self.1.emplace(u_1)?;
        Ok(unsafe { uninit.assume_init_mut() })
    }
}

#[allow(dead_code)]
impl<__Last: Emplacer<FlatVec<u8, u16>>> Emplacer<UnsizedEnum> for UnsizedEnumInitC<__Last> {
    fn emplace(self, uninit: &mut MaybeUninitUnsized<UnsizedEnum>) -> Result<&mut UnsizedEnum, Error> {
        let bytes = uninit.as_mut_bytes();
        unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_mut_bytes_unchecked(bytes) }
            .as_mut_sized()
            .write(UnsizedEnumTag::C);
        let iter = iter::MutIter::new(
            unsafe { bytes.get_unchecked_mut(UnsizedEnum::DATA_OFFSET..) },
            iter::type_list!(u8, FlatVec<u8, u16>),
        )
        .map_err(|e| e.offset(UnsizedEnum::DATA_OFFSET))?;
        let (iter, u_a) = iter.next();
        self.a.emplace(u_a)?;
        let u_b = iter.finalize();
        self.b.emplace(u_b)?;
        Ok(unsafe { uninit.assume_init_mut() })
    }
}

impl UnsizedEnum {
    const DATA_OFFSET: usize = ceil_mul(u8::SIZE, Self::ALIGN);
    const LAST_FIELD_OFFSETS: [usize; 3] = [
        0,
        ceil_mul(iter::fold_size!(0; u8), u16::ALIGN),
        ceil_mul(iter::fold_size!(0; u8), FlatVec::<u8, u16>::ALIGN),
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
                let iter = unsafe { iter::RefIter::new_unchecked(&self.data, iter::type_list!(u8, u16)) };
                let (iter, value) = iter.next();
                let b0 = unsafe { value.assume_init() };
                let value = iter.finalize();
                let b1 = unsafe { value.assume_init() };
                UnsizedEnumRef::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter = unsafe { iter::RefIter::new_unchecked(&self.data, iter::type_list!(u8, FlatVec<u8, u16>)) };
                let (iter, value) = iter.next();
                let a = unsafe { value.assume_init() };
                let value = iter.finalize();
                let b = unsafe { value.assume_init() };
                UnsizedEnumRef::C { a, b }
            }
        }
    }
    pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
        match self.tag {
            UnsizedEnumTag::A => UnsizedEnumMut::A,
            UnsizedEnumTag::B => {
                let iter = unsafe { iter::MutIter::new_unchecked(&mut self.data, iter::type_list!(u8, u16)) };
                let (iter, value) = iter.next();
                let b0 = unsafe { value.assume_init_mut() };
                let value = iter.finalize();
                let b1 = unsafe { value.assume_init_mut() };
                UnsizedEnumMut::B(b0, b1)
            }
            UnsizedEnumTag::C => {
                let iter = unsafe { iter::MutIter::new_unchecked(&mut self.data, iter::type_list!(u8, FlatVec<u8, u16>)) };
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
    const MIN_SIZE: usize =
        Self::DATA_OFFSET + min(Self::DATA_MIN_SIZES[0], min(Self::DATA_MIN_SIZES[1], Self::DATA_MIN_SIZES[2]));

    fn size(&self) -> usize {
        ceil_mul(
            Self::DATA_OFFSET
                + match self.tag {
                    UnsizedEnumTag::A => 0,
                    UnsizedEnumTag::B => unsafe {
                        iter::RefIter::new_unchecked(&self.data, iter::type_list!(u8, u16)).fold_size(0)
                    },
                    UnsizedEnumTag::C => unsafe {
                        iter::RefIter::new_unchecked(&self.data, iter::type_list!(u8, FlatVec<u8, u16>)).fold_size(0)
                    },
                },
            Self::ALIGN,
        )
    }
}

unsafe impl FlatUnsized for UnsizedEnum {
    type AlignAs = UnsizedEnumAlignAs;

    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize {
        floor_mul(this.as_bytes().len() - Self::DATA_OFFSET, Self::ALIGN)
    }
    fn bytes_len(this: &Self) -> usize {
        Self::DATA_OFFSET + this.data.len()
    }

    impl_unsized_uninit_cast!();
}

impl FlatCheck for UnsizedEnum {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error> {
        let bytes = this.as_bytes();
        let tag = unsafe { MaybeUninitUnsized::<UnsizedEnumTag>::from_bytes_unchecked(bytes) };
        UnsizedEnumTag::validate(tag)?;
        let bytes = unsafe { bytes.get_unchecked(Self::DATA_OFFSET..) };
        match unsafe { tag.assume_init() } {
            UnsizedEnumTag::A => Ok(()),
            UnsizedEnumTag::B => unsafe { iter::RefIter::new_unchecked(bytes, iter::type_list!(u8, u16)).validate_all() },
            UnsizedEnumTag::C => unsafe {
                iter::RefIter::new_unchecked(bytes, iter::type_list!(u8, FlatVec<u8, u16>)).validate_all()
            },
        }
        .map_err(|e| e.offset(Self::DATA_OFFSET))
        .map(|_| unsafe { this.assume_init() })
    }
}

impl FlatDefault for UnsizedEnum {
    type Emplacer = UnsizedEnumInitA;
    fn default_emplacer() -> Self::Emplacer {
        UnsizedEnumInitA
    }
}

unsafe impl Flat for UnsizedEnum {}

generate_tests!();
