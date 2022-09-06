use super::tests::generate_tests;
use flatty::{
    iter::{prelude::*, RefIter},
    mem::Muu,
    prelude::*,
    type_list, Error,
};

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
        unsafe { RefIter::new_unchecked(this.as_bytes(), type_list!(u8, u16, u32, [u64; 4])) }
            .validate_all()
    }
}

unsafe impl Flat for SizedStruct {}

generate_tests!();
