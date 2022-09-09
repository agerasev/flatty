#![allow(dead_code)]

use flatty::{make_flat, FlatVec};

#[make_flat(default = false)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

#[make_flat(enum_type = "u8", default = false)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum SizedEnum {
    A,
    B(u16, u8),
    C { a: u8, b: u16 },
    D(u32),
}

#[make_flat(sized = false, default = false)]
#[derive(Debug, PartialEq, Eq)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64, u32>,
}

#[make_flat(sized = false, enum_type = "u8", default = false)]
enum UnsizedEnum {
    A,
    B(u8, u16),
    C { a: u8, b: FlatVec<u8, u16> },
}
