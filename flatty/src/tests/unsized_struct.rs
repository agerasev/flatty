use crate::{
    base::{Flat, FlatBase, FlatInit, InterpretError},
    sized::FlatSized,
    util::upper_multiple,
    FlatVec,
};
use core::slice::{from_raw_parts, from_raw_parts_mut};

#[repr(C)]
pub struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64>,
}

impl UnsizedStruct {
    const LAST_FIELD_OFFSET: usize = upper_multiple(
        upper_multiple(<u8 as FlatSized>::SIZE, u16::ALIGN) + u16::SIZE,
        <FlatVec<u64>>::ALIGN,
    );
}

#[allow(dead_code)]
pub struct UnsizedStructAlignAs {
    a: <u8 as FlatBase>::AlignAs,
    b: <u16 as FlatBase>::AlignAs,
    c: <FlatVec<u64> as FlatBase>::AlignAs,
}

impl FlatBase for UnsizedStruct {
    type AlignAs = UnsizedStructAlignAs;

    const MIN_SIZE: usize = Self::LAST_FIELD_OFFSET + <FlatVec<u64>>::MIN_SIZE;
    fn size(&self) -> usize {
        Self::LAST_FIELD_OFFSET + self.c.size()
    }

    fn _ptr_metadata(mem: &[u8]) -> usize {
        <FlatVec<u64>>::_ptr_metadata(&mem[Self::LAST_FIELD_OFFSET..])
    }
}

#[derive(Default)]
pub struct UnsizedStructInit {
    a: <u8 as FlatInit>::Init,
    b: <u16 as FlatInit>::Init,
    c: <FlatVec<u64> as FlatInit>::Init,
}

impl FlatInit for UnsizedStruct {
    type Init = UnsizedStructInit;

    unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
        let mut offset = 0;
        <u8 as FlatInit>::init_unchecked(&mut mem[offset..], init.a);
        offset = upper_multiple(offset + <u8>::SIZE, u16::ALIGN);
        <u16 as FlatInit>::init_unchecked(&mut mem[offset..], init.b);
        offset = upper_multiple(offset + <u16>::SIZE, <FlatVec<u64>>::ALIGN);
        <FlatVec<u64> as FlatInit>::init_unchecked(&mut mem[offset..], init.c);
        Self::interpret_mut_unchecked(mem)
    }

    fn pre_validate(mem: &[u8]) -> Result<(), InterpretError> {
        let mut offset = 0;
        <u8 as FlatInit>::pre_validate(&mem[offset..])?;
        offset = upper_multiple(offset + <u8>::SIZE, u16::ALIGN);
        <u16 as FlatInit>::pre_validate(&mem[offset..])?;
        offset = upper_multiple(offset + <u16>::SIZE, <FlatVec<u64>>::ALIGN);
        <FlatVec<u64> as FlatInit>::pre_validate(&mem[offset..])?;
        Ok(())
    }
    fn post_validate(&self) -> Result<(), InterpretError> {
        self.a.post_validate()?;
        self.b.post_validate()?;
        self.c.post_validate()?;
        Ok(())
    }

    unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
        let slice = from_raw_parts(mem.as_ptr(), Self::_ptr_metadata(mem));
        &*(slice as *const [_] as *const Self)
    }
    unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        let slice = from_raw_parts_mut(mem.as_mut_ptr(), Self::_ptr_metadata(mem));
        &mut *(slice as *mut [_] as *mut Self)
    }
}

unsafe impl Flat for UnsizedStruct {}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let unsized_struct = UnsizedStruct::init(
        mem.as_mut_slice(),
        UnsizedStructInit {
            a: 200,
            b: 40000,
            c: <FlatVec<u64> as FlatInit>::Init::Empty,
        },
    )
    .unwrap();

    assert_eq!(unsized_struct.a, 200);
    assert_eq!(unsized_struct.b, 40000);
    assert_eq!(unsized_struct.c.len(), 0);

    for i in 0.. {
        if unsized_struct.c.push(i).is_err() {
            break;
        }
    }

    assert_eq!(unsized_struct.a, 200);
    assert_eq!(unsized_struct.b, 40000);
    assert_eq!(unsized_struct.c.len(), 4);
    assert_eq!(unsized_struct.c.as_slice(), [0, 1, 2, 3]);
}
