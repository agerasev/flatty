use flatty::{make_flat, portable::le, FlatVec};

#[make_flat(portable = true)]
struct PortableStruct {
    a: u8,
    b: le::U16,
    c: le::U32,
    d: [le::U64; 4],
}

#[make_flat(sized = false, portable = true)]
struct PortableUnsizedStruct {
    a: le::U16,
    b: FlatVec<le::U32, le::U16>,
}

#[make_flat(sized = false, portable = true)]
enum PortableEnum {
    A,
    B(le::F32, PortableStruct),
    C(PortableUnsizedStruct),
}
