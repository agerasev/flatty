use crate::{self as flatty, FlatInit, FlatSized};

#[derive(FlatSized)]
struct S {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

#[test]
fn init() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let s = S::init(
        m.as_mut_slice(),
        S {
            a: 200,
            b: 40000,
            c: 2000000000,
            d: [1, 2, 3, 4],
        },
    )
    .unwrap();

    assert_eq!(s.a, 200);
    assert_eq!(s.b, 40000);
    assert_eq!(s.c, 2000000000);
    assert_eq!(s.d, [1, 2, 3, 4]);
}
