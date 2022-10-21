use super::tests::generate_tests;
use flatty::{flat, FlatVec};

#[flat(sized = false)]
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
