use super::tests::generate_tests;
use flatty::make_flat;

#[make_flat(enum_type = "u8")]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
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
