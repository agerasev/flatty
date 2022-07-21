use crate::{
    base::{Flat, FlatBase, FlatInit, FlatUnsized},
    error::InterpretError,
    sized::FlatSized,
    utils::{max, min},
    FlatVec,
};
use core::slice::{from_raw_parts, from_raw_parts_mut};

#[repr(u8)]
pub enum UnsizedEnumState {
    A,
    B,
    C,
}

#[repr(C)]
pub struct UnsizedEnum {
    state: UnsizedEnumState,
    _align: [<Self as FlatUnsized>::AlignAs; 0],
    data: [u8],
}

pub enum UnsizedEnumRef<'a> {
    A,
    B(&'a i32),
    C(&'a FlatVec<u8>),
}

pub enum UnsizedEnumMut<'a> {
    A,
    B(&'a mut i32),
    C(&'a mut FlatVec<u8>),
}

impl UnsizedEnum {
    const DATA_OFFSET: usize = max(u8::SIZE, Self::ALIGN);

    pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
        match self.state {
            UnsizedEnumState::A => UnsizedEnumRef::A,
            UnsizedEnumState::B => {
                UnsizedEnumRef::B(unsafe { i32::interpret_unchecked(&self.data) })
            }
            UnsizedEnumState::C => {
                UnsizedEnumRef::C(unsafe { FlatVec::<u8>::interpret_unchecked(&self.data) })
            }
        }
    }

    pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
        match self.state {
            UnsizedEnumState::A => UnsizedEnumMut::A,
            UnsizedEnumState::B => {
                UnsizedEnumMut::B(unsafe { i32::interpret_mut_unchecked(&mut self.data) })
            }
            UnsizedEnumState::C => {
                UnsizedEnumMut::C(unsafe { FlatVec::<u8>::interpret_mut_unchecked(&mut self.data) })
            }
        }
    }
}

#[allow(dead_code)]
#[repr(u8)]
pub enum UnsizedEnumAlignAs {
    A,
    B(<i32 as FlatUnsized>::AlignAs),
    C(<FlatVec<u8> as FlatUnsized>::AlignAs),
}

impl FlatBase for UnsizedEnum {
    const ALIGN: usize = max(u8::ALIGN, max(i32::ALIGN, <FlatVec<u8>>::ALIGN));

    const MIN_SIZE: usize = Self::DATA_OFFSET + min(0, min(i32::MIN_SIZE, FlatVec::<u8>::MIN_SIZE));
    fn size(&self) -> usize {
        Self::DATA_OFFSET
            + match self.as_ref() {
                UnsizedEnumRef::A => 0,
                UnsizedEnumRef::B(r) => r.size(),
                UnsizedEnumRef::C(r) => r.size(),
            }
    }
}

impl FlatUnsized for UnsizedEnum {
    type AlignAs = UnsizedEnumAlignAs;

    fn ptr_metadata(mem: &[u8]) -> usize {
        mem.len() - Self::DATA_OFFSET
    }
}

pub enum UnsizedEnumInit {
    A,
    B(<i32 as FlatInit>::Init),
    C(<FlatVec<u8> as FlatInit>::Init),
}

impl FlatInit for UnsizedEnum {
    type Init = UnsizedEnumInit;
    unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
        let self_ = Self::interpret_mut_unchecked(mem);
        match init {
            UnsizedEnumInit::A => {
                self_.state = UnsizedEnumState::A;
            }
            UnsizedEnumInit::B(inner_init) => {
                self_.state = UnsizedEnumState::B;
                i32::init_unchecked(&mut self_.data, inner_init);
            }
            UnsizedEnumInit::C(inner_init) => {
                self_.state = UnsizedEnumState::C;
                <FlatVec<u8>>::init_unchecked(&mut self_.data, inner_init);
            }
        }
        self_
    }

    fn pre_validate(mem: &[u8]) -> Result<(), InterpretError> {
        if *u8::interpret(mem).unwrap() >= 3 {
            Err(InterpretError::InvalidState)
        } else {
            Ok(())
        }
    }
    fn post_validate(&self) -> Result<(), InterpretError> {
        match &self.state {
            UnsizedEnumState::A => Ok(()),
            UnsizedEnumState::B => {
                if self.data.len() < i32::MIN_SIZE {
                    return Err(InterpretError::InsufficientSize);
                }
                i32::pre_validate(&self.data)?;
                if let UnsizedEnumRef::B(inner) = self.as_ref() {
                    inner.post_validate()
                } else {
                    unreachable!();
                }
            }
            UnsizedEnumState::C => {
                if self.data.len() < FlatVec::<u8>::MIN_SIZE {
                    return Err(InterpretError::InsufficientSize);
                }
                <FlatVec<u8>>::pre_validate(&self.data)?;
                if let UnsizedEnumRef::C(inner) = self.as_ref() {
                    inner.post_validate()
                } else {
                    unreachable!();
                }
            }
        }
    }

    unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
        let slice = from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
        &*(slice as *const [_] as *const Self)
    }
    unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
        let slice = from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
        &mut *(slice as *mut [_] as *mut Self)
    }
}

unsafe impl Flat for UnsizedEnum {}

#[test]
fn init_a() {
    let mut mem = vec![0u8; 4];
    let unsized_enum = UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::A).unwrap();

    match unsized_enum.as_ref() {
        UnsizedEnumRef::A => (),
        _ => panic!(),
    }
}

#[test]
fn init_b() {
    let mut mem = vec![0u8; 8];
    let unsized_enum = UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::B(42)).unwrap();

    match unsized_enum.as_ref() {
        UnsizedEnumRef::B(x) => assert_eq!(*x, 42),
        _ => panic!(),
    }
}

#[test]
fn init_c() {
    let mut mem = vec![0u8; 12];
    let unsized_enum = UnsizedEnum::init(
        mem.as_mut_slice(),
        UnsizedEnumInit::C(<FlatVec<u8> as FlatInit>::Init::default()),
    )
    .unwrap();

    match unsized_enum.as_mut() {
        UnsizedEnumMut::C(v) => {
            assert_eq!(v.len(), 0);
            for i in 0.. {
                if v.push(i).is_err() {
                    break;
                }
            }
        }
        _ => panic!(),
    }
    match unsized_enum.as_ref() {
        UnsizedEnumRef::C(v) => {
            assert_eq!(v.len(), 4);
            for (i, x) in v.as_slice().iter().enumerate() {
                assert_eq!(i as u8, *x);
            }
        }
        _ => panic!(),
    }
}

#[test]
fn init_err() {
    let mut mem = vec![0u8; 1];
    let res = UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::A);
    assert_eq!(res.err().unwrap(), InterpretError::InsufficientSize);
}
