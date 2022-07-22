use flatty::{FlatUnsized, FlatVec};

#[derive(FlatUnsized)]
#[repr(C)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    d: FlatVec<u64>,
}
