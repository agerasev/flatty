use flatty::{make_flat, FlatInit, FlatSized};

#[make_flat(enum_type = "u8")]
#[derive(Default)]
enum SizedEnum {
    #[default]
    A,
    B(i32),
    C(u8, u16),
}

#[test]
fn init_a() {
    let mut m = vec![0u8; 4 + 4];
    let e = SizedEnum::init_default(m.as_mut_slice()).unwrap();

    if let SizedEnum::A = e {
    } else {
        panic!();
    }

    assert_eq!(m[0], 0);
}

#[test]
fn init_b() {
    let mut m = vec![0u8; 4 + 4];
    let e = SizedEnum::init(m.as_mut_slice(), SizedEnum::B(0x12345678)).unwrap();

    if let SizedEnum::B(a) = e {
        assert_eq!(*a, 0x12345678);
    } else {
        panic!();
    }

    assert_eq!(m[0], 1);
    assert_eq!(&m[4..], [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn init_c() {
    let mut m = vec![0u8; 4 + 4];
    let e = SizedEnum::init(m.as_mut_slice(), SizedEnum::C(0xab, 0xabcd)).unwrap();

    if let SizedEnum::C(a, b) = e {
        assert_eq!(*a, 0xab);
        assert_eq!(*b, 0xabcd);
    } else {
        panic!();
    }

    assert_eq!(m[0], 2);
    assert_eq!(m[4], 0xab);
    assert_eq!(&m[6..], [0xcd, 0xab]);
}

#[test]
fn interpret_c() {
    let m = vec![2, 0, 0, 0, 0xab, 0, 0xcd, 0xab];
    let s = SizedEnum::interpret(m.as_slice()).unwrap();

    if let SizedEnum::C(a, b) = s {
        assert_eq!(*a, 0xab);
        assert_eq!(*b, 0xabcd);
    } else {
        panic!();
    }
}
