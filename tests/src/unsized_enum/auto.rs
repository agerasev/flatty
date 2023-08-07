use super::tests::generate_tests;
use flatty::{flat, FlatVec};

#[flat(sized = false, default = true)]
enum UnsizedEnum {
    #[default]
    A,
    B(u8, u16),
    C {
        offset: u32,
        bytes: FlatVec<u8, u16>,
    },
}

generate_tests!();
