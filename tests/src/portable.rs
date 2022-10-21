#![allow(dead_code)]

use flatty::{flat, portable::le, FlatVec};

#[flat(portable = true, default = true)]
struct PortableStruct {
    a: u8,
    b: le::U16,
    c: le::U32,
    d: [le::U64; 4],
}

#[flat(portable = true, default = true)]
enum PortableEnum {
    #[default]
    A,
    B(le::F32, PortableStruct),
    C(PortableStruct),
}

#[flat(sized = false, portable = true, default = true)]
struct PortableUnsizedStruct {
    a: le::U16,
    b: FlatVec<le::U32, le::U16>,
}

#[flat(sized = false, portable = true, default = true)]
enum PortableUnsizedEnum {
    #[default]
    A,
    B(le::F32, PortableStruct),
    C(PortableUnsizedStruct),
}
