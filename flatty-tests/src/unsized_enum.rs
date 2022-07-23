use flatty::{make_flat, FlatVec};

#[make_flat(sized = false, enum_type = "u8")]
enum UnsizedEnum {
    A,
    B(i32),
    C(FlatVec<u8>),
}
