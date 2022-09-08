use super::tests::generate_tests;
use flatty::{make_flat, FlatVec};

#[make_flat(sized = false, enum_type = "u8")]
enum UnsizedEnum {
    #[default]
    A,
    B(u8, u16),
    C {
        a: u8,
        b: FlatVec<u8, u16>,
    },
}

generate_tests!();
