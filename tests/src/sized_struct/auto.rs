use super::tests::generate_tests;
use flatty::flat;

#[flat]
#[derive(Clone, Debug, PartialEq, Eq)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

generate_tests!();
