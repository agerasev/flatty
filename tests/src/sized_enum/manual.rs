use super::tests::generate_tests;
use flatty::{
    iter::{prelude::*, type_list, RefIter},
    mem::MaybeUninitUnsized,
    prelude::*,
    utils::ceil_mul,
    Error, ErrorKind,
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

impl FlatCast for SizedEnum {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<(), Error> {
        let bytes = this.as_bytes();
        let tag = unsafe { MaybeUninitUnsized::<u8>::from_bytes_unchecked(bytes) };
        u8::validate(tag)?;
        let data_offset: usize = ceil_mul(u8::SIZE, Self::ALIGN);
        let bytes = unsafe { bytes.get_unchecked(data_offset..) };
        match unsafe { tag.assume_init_ref() } {
            0 => Ok(()),
            1 => unsafe { RefIter::new_unchecked(bytes, type_list!(u16, u8)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            2 => unsafe { RefIter::new_unchecked(bytes, type_list!(u8, u16)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            3 => unsafe { RefIter::new_unchecked(bytes, type_list!(u32)) }
                .validate_all()
                .map_err(|e| e.offset(data_offset)),
            _ => Err(Error {
                kind: ErrorKind::InvalidEnumState,
                pos: 0,
            }),
        }
    }
}

unsafe impl Flat for SizedEnum {}

generate_tests!();
