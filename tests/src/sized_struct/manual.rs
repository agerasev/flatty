use super::tests::generate_tests;
use flatty::{
    mem::MaybeUninitUnsized,
    prelude::*,
    type_list,
    utils::iter::{self, prelude::*},
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

impl FlatCheck for SizedStruct {
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error> {
        unsafe { iter::RefIter::new_unchecked(this.as_bytes(), type_list!(u8, u16, u32, [u64; 4])) }.validate_all()?;
        Ok(unsafe { this.assume_init() })
    }
}

unsafe impl Flat for SizedStruct {}

generate_tests!();
