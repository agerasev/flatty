use super::tests::generate_tests;
use flatty::flat;

#[flat]
#[derive(Clone, Debug, PartialEq, Eq)]
enum SizedEnum {
    #[default]
    A,
    B(u16, u8),
    C {
        a: u8,
        b: u16,
    },
    D(u32),
}

generate_tests!();
