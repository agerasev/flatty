use flatty::{FlatInit, FlatSized};

#[derive(FlatSized, Default)]
#[repr(C)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

#[test]
fn init() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let s = SizedStruct::init(
        m.as_mut_slice(),
        SizedStruct {
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

#[test]
fn init_default() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let s = SizedStruct::init_default(m.as_mut_slice()).unwrap();

    assert_eq!(s.a, u8::default());
    assert_eq!(s.b, u16::default());
    assert_eq!(s.c, u32::default());
    assert_eq!(s.d, <[u64; 4]>::default());
}

#[test]
fn interpret() {
    let m = (0..4).fold(
        vec![0x12, 0xff, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12],
        |mut a, i| {
            a.extend([i + 1, 0, 0, 0, 0, 0, 0, 0].into_iter());
            a
        },
    );
    let s = SizedStruct::interpret(m.as_slice()).unwrap();

    assert_eq!(s.a, 0x12);
    assert_eq!(s.b, 0x1234);
    assert_eq!(s.c, 0x12345678);
    assert_eq!(s.d, [1, 2, 3, 4]);
}
