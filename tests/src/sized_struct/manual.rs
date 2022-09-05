use super::tests::generate_tests;
use flatty::{mem::Muu, prelude::*, utils::ceil_mul, Error};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[repr(C)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

impl FlatCast for SizedStruct {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let mut pos = 0;
        let bytes = this.as_bytes();

        u8::validate(unsafe { Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u8::SIZE, u16::ALIGN);

        u16::validate(unsafe { Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u16::SIZE, u32::ALIGN);

        u32::validate(unsafe { Muu::<u32>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u32::SIZE, <[u64; 4]>::ALIGN);

        <[u64; 4]>::validate(unsafe {
            Muu::<[u64; 4]>::from_bytes_unchecked(bytes.get_unchecked(pos..))
        })
        .map_err(|e| e.offset(pos))?;

        Ok(())
    }
}

unsafe impl Flat for SizedStruct {}

generate_tests!();
