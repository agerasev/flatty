use super::tests::generate_tests;
use flatty::flat;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[flat]
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
