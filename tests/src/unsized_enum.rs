use core::{
    mem::{align_of_val, size_of_val},
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};
use flatty::{
    make_flat,
    mem::Muu,
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
    _align: [AlignAs; 0],
    tag: UnsizedEnumTag,
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
struct AlignAs(u8, u8, u16, u8, <FlatVec<u8, u16> as FlatUnsized>::AlignAs);

impl UnsizedEnum {
    const DATA_OFFSET: usize = ceil_mul(u8::SIZE, Self::ALIGN);
    const DATA_MIN_SIZES: [usize; 3] = [
        0,
        ceil_mul(ceil_mul(u8::SIZE, u16::ALIGN) + u16::SIZE, Self::ALIGN),
        ceil_mul(
            ceil_mul(u8::SIZE, FlatVec::<u8, u16>::ALIGN) + FlatVec::<u8, u16>::MIN_SIZE,
            Self::ALIGN,
        ),
    ];

    pub fn tag(&self) -> UnsizedEnumTag {
        self.tag
    }
    pub fn set_tag(&mut self, tag: UnsizedEnumTag) {
        self.tag = tag;
        unimplemented!();
    }

    pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
        let pos = 0;
        unsafe {
            match self.tag {
                UnsizedEnumTag::A => UnsizedEnumRef::A,
                UnsizedEnumTag::B => UnsizedEnumRef::B(
                    {
                        pos = ceil_mul(pos, u8::ALIGN);
                        let mu = Muu::<u8>::from_bytes_unchecked(self.data.get_unchecked(pos..));
                        pos += u8::SIZE;
                        &*mu.as_ptr()
                    },
                    {
                        pos = ceil_mul(pos, u8::ALIGN);
                        let mu = Muu::<u8>::from_bytes_unchecked(self.data.get_unchecked(pos..));
                        pos += u8::SIZE;
                        &*mu.as_ptr()
                    },
                ),
                UnsizedEnumTag::C => {
                    size = ceil_mul(size, u8::ALIGN) + u8::SIZE;
                    size = ceil_mul(size, FlatVec::<u8, u16>::ALIGN) + b.size();
                }
            }
        }
    }
    pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
        unimplemented!();
    }
}

unsafe impl FlatBase for UnsizedEnum {
    const ALIGN: usize = max(max(u8::ALIGN, u16::ALIGN), FlatVec::<u8, u16>::ALIGN);
    const MIN_SIZE: usize = Self::DATA_OFFSET
        + min(
            min(Self::DATA_MIN_SIZES[0], Self::DATA_MIN_SIZES[1]),
            Self::DATA_MIN_SIZES[2],
        );

    fn size(&self) -> usize {
        let mut size = ceil_mul(u8::SIZE, Self::ALIGN);
        match self.as_ref() {
            UnsizedEnumRef::A => {}
            UnsizedEnumRef::B(_b0, b1) => {
                size = ceil_mul(size, u8::ALIGN) + u8::SIZE;
                size = ceil_mul(size, u16::ALIGN) + b1.size();
            }
            UnsizedEnumRef::C { a: _a, b } => {
                size = ceil_mul(size, u8::ALIGN) + u8::SIZE;
                size = ceil_mul(size, FlatVec::<u8, u16>::ALIGN) + b.size();
            }
        }

        ceil_mul(size, Self::ALIGN)
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

unsafe impl FlatUnsized for UnsizedEnum {
    type AlignAs = AlignAs;

    fn ptr_metadata(bytes: &[u8]) -> Option<usize> {
        Some(floor_mul(bytes.len() - Self::DATA_OFFSET, Self::ALIGN))
    }
}

impl FlatCast for UnsizedEnum {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let bytes = this.as_bytes();
        let tag = unsafe { Muu::<u8>::from_bytes_unchecked(bytes) };
        u8::validate(tag)?;
        let bytes = unsafe { bytes.get_unchecked(Self::DATA_OFFSET..) };
        match unsafe { *tag.as_ptr() } {
            0 => Ok(()),
            1 => {
                u8::validate(unsafe {
                    Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;
                pos += ceil_mul(pos + u8::SIZE, u16::ALIGN);

                u16::validate(unsafe {
                    Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;

                Ok(())
            }
            2 => {
                u8::validate(unsafe {
                    Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;
                pos += ceil_mul(pos + u8::SIZE, u16::ALIGN);

                FlatVec::<u8, u16>::validate(unsafe {
                    Muu::<FlatVec<u8, u16>>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;
                Ok(())
            }
            _ => Err(Error {
                kind: ErrorKind::InvalidEnumState,
                pos: 0,
            }),
        }
    }
}

impl FlatDefault for UnsizedEnum {
    fn init_default(this: &mut Muu<Self>) -> Result<(), Error> {
        let mut pos = 0;
        let bytes = this.as_mut_bytes();

        u8::init_default(unsafe {
            Muu::<u8>::from_mut_bytes_unchecked(bytes.get_unchecked_mut(pos..))
        })
        .map_err(|e| e.offset(pos))?;
        pos = ceil_mul(pos + u8::SIZE, u16::ALIGN);

        u16::init_default(unsafe {
            Muu::<u16>::from_mut_bytes_unchecked(bytes.get_unchecked_mut(pos..))
        })
        .map_err(|e| e.offset(pos))?;
        pos = ceil_mul(pos + u16::SIZE, FlatVec::<u8, u16>::ALIGN);

        FlatVec::<u8, u16>::init_default(unsafe {
            Muu::<FlatVec<u8, u16>>::from_mut_bytes_unchecked(bytes.get_unchecked_mut(pos..))
        })
        .map_err(|e| e.offset(pos))?;

        Ok(())
    }
}

unsafe impl Flat for UnsizedEnum {}

/*
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
    let ue =
        UnsizedEnum::placement_new(mem.as_mut_slice(), &UnsizedEnumDyn::B(0xab, 0xcdef)).unwrap();
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
    let ue = UnsizedEnum::placement_new(
        mem.as_mut_slice(),
        &UnsizedEnumDyn::C {
            a: 0xab,
            b: vec![0x12, 0x34, 0x56, 0x78],
        },
    )
    .unwrap();
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
    let res = UnsizedEnum::placement_new(mem.as_mut_slice(), &UnsizedEnumDyn::A);
    assert_eq!(res.err().unwrap(), Error::InsufficientSize);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 6 + 8 * 2 + 1];
    let us = UnsizedEnum::placement_new(
        mem.as_mut_slice(),
        &UnsizedEnumDyn::C {
            a: 0xab,
            b: Vec::new(),
        },
    )
    .unwrap();

    if let UnsizedEnumMut::C { b, .. } = us.as_mut() {
        for i in 0.. {
            if b.push(i).is_err() {
                break;
            }
        }
    } else {
        panic!();
    }

    assert_eq!(UnsizedEnum::DATA_OFFSET, 2);
    assert_eq!(align_of_val(us), <UnsizedEnum as FlatBase>::ALIGN);
    assert_eq!(size_of_val(us), us.size());
    assert_eq!(us.size(), mem.len() - 1);
}
*/
