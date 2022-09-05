use super::tests::generate_tests;
use flatty::{mem::Muu, prelude::*, utils::ceil_mul, Error, ErrorKind};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[repr(C, u8)]
enum SizedEnum {
    #[default]
    A,
    B(u16, u8),
    C {
        a: u8,
        b: u16,
    },
    D(u32),
}

impl FlatCast for SizedEnum {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let bytes = this.as_bytes();
        let mut pos = 0;
        let tag = unsafe { Muu::<u8>::from_bytes_unchecked(bytes) };
        u8::validate(tag)?;
        pos += ceil_mul(pos + u8::SIZE, Self::ALIGN);
        match unsafe { *tag.as_ptr() } {
            0 => Ok(()),
            1 => {
                u16::validate(unsafe {
                    Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;
                pos += ceil_mul(pos + u16::SIZE, u8::ALIGN);

                u8::validate(unsafe {
                    Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..))
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

                u16::validate(unsafe {
                    Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                })
                .map_err(|e| e.offset(pos))?;
                Ok(())
            }
            3 => {
                u32::validate(unsafe {
                    Muu::<u32>::from_bytes_unchecked(bytes.get_unchecked(pos..))
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

unsafe impl Flat for SizedEnum {}

generate_tests!();
