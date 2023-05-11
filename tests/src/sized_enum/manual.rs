use super::tests::generate_tests;
use flatty::{
    error::{Error, ErrorKind},
    prelude::*,
    utils::{
        ceil_mul,
        iter::{self, prelude::*},
    },
};

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

unsafe impl FlatValidate for SizedEnum {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        u8::validate_unchecked(bytes)?;
        let tag = u8::from_bytes_unchecked(bytes);

        let data_offset: usize = ceil_mul(u8::SIZE, Self::ALIGN);
        let bytes = unsafe { bytes.get_unchecked(data_offset..) };
        match tag {
            0 => Ok(()),
            1 => unsafe { iter::DataIter::new_unchecked(bytes, iter::type_list!(u16, u8)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            2 => unsafe { iter::DataIter::new_unchecked(bytes, iter::type_list!(u8, u16)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            3 => unsafe { iter::DataIter::new_unchecked(bytes, iter::type_list!(u32)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            _ => Err(Error {
                kind: ErrorKind::InvalidEnumTag,
                pos: 0,
            }),
        }
    }
}

unsafe impl Flat for SizedEnum {}

generate_tests!();
