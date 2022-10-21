//use super::tests::generate_tests;
use flatty::{make_flat, FlatVec};

#[make_flat(sized = false)]
#[derive(Debug, PartialEq, Eq)]
pub struct UnsizedStruct {
    pub a: u8,
    b: u16,
    c: FlatVec<u64, u32>,
}

//generate_tests!();
