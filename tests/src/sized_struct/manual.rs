use super::tests::generate_tests;
use flatty::{
    prelude::*,
    utils::iter::{self, prelude::*, type_list},
    Error,
};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[repr(C)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

unsafe impl FlatValidate for SizedStruct {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        unsafe { iter::BytesIter::new_unchecked(bytes, type_list!(u8, u16, u32, [u64; 4])) }.validate_all()?;
        Ok(())
    }
}

unsafe impl Flat for SizedStruct {}

generate_tests!();
