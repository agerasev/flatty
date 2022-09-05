use core::{
    mem::{align_of_val, size_of_val},
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};
use flatty::{
    make_flat,
    mem::Muu,
    prelude::*,
    utils::{ceil_mul, max},
    Error, FlatVec,
};

//#[make_flat(sized = false)]
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64>,
}

#[repr(C)]
struct AlignAs(u8, u16, <FlatVec<u64> as FlatUnsized>::AlignAs);

unsafe impl FlatBase for UnsizedStruct {
    const ALIGN: usize = max(max(u8::ALIGN, u16::ALIGN), FlatVec::<u64>::ALIGN);
    const MIN_SIZE: usize = ceil_mul(
        ceil_mul(
            ceil_mul(u8::SIZE, u16::ALIGN) + u16::SIZE,
            FlatVec::<u64>::ALIGN,
        ) + FlatVec::<u64>::MIN_SIZE,
        Self::ALIGN,
    );

    fn size(&self) -> usize {
        let mut size = 0;

        size = ceil_mul(size, u8::ALIGN) + u8::SIZE;
        size = ceil_mul(size, u16::ALIGN) + u16::SIZE;
        size = ceil_mul(size, FlatVec::<u64>::ALIGN) + self.c.size();

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

unsafe impl FlatUnsized for UnsizedStruct {
    type AlignAs = AlignAs;

    fn ptr_metadata(bytes: &[u8]) -> Option<usize> {
        FlatVec::<u64>::ptr_metadata(&bytes[(Self::MIN_SIZE - FlatVec::<u64>::MIN_SIZE)..])
    }
}

impl FlatCast for UnsizedStruct {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let mut pos = 0;
        let bytes = this.as_bytes();

        u8::validate(unsafe { Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos = ceil_mul(pos + u8::SIZE, u16::ALIGN);

        u16::validate(unsafe { Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos = ceil_mul(pos + u16::SIZE, FlatVec::<u64>::ALIGN);

        FlatVec::<u64>::validate(unsafe {
            Muu::<FlatVec<u64>>::from_bytes_unchecked(bytes.get_unchecked(pos..))
        })
        .map_err(|e| e.offset(pos))?;

        Ok(())
    }
}

impl FlatDefault for UnsizedStruct {
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
        pos = ceil_mul(pos + u16::SIZE, FlatVec::<u64>::ALIGN);

        FlatVec::<u64>::init_default(unsafe {
            Muu::<FlatVec<u64>>::from_mut_bytes_unchecked(bytes.get_unchecked_mut(pos..))
        })
        .map_err(|e| e.offset(pos))?;

        Ok(())
    }
}

unsafe impl Flat for UnsizedStruct {}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 8 * 4];
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
    let mut mem = vec![0u8; 16 + 8 * 4];
    let us = UnsizedStruct::placement_default(mem.as_mut_slice()).unwrap();

    assert_eq!(us.size(), 16);
    assert_eq!(us.a, 0);
    assert_eq!(us.b, 0);
    assert_eq!(us.c.len(), 0);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 16 + 8 * 4];
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
    let mut mem_ab = vec![0u8; 16 + 8 * 4];
    let mut mem_c = vec![0u8; 16 + 8 * 3];
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
