use flatty::{make_flat, prelude::*, FlatVec};

#[make_flat(sized = false)]
#[derive(Debug, PartialEq, Eq)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64, u32>,
}
