use flatty::{FlatInit, FlatSized};

#[derive(FlatSized, Default)]
#[repr(C, u8)]
enum E {
    #[default]
    A,
    B(i32),
    C(u8, u16),
}

#[test]
fn init_a() {
    let mut m = vec![0u8; 4 + 4];
    let e = E::init_default(m.as_mut_slice()).unwrap();

    if let E::A = e {
    } else {
        panic!();
    }

    assert_eq!(m[0], 0);
}

#[test]
fn init_b() {
    let mut m = vec![0u8; 4 + 4];
    let e = E::init(m.as_mut_slice(), E::B(0x12345678)).unwrap();

    if let E::B(a) = e {
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
    let e = E::init(m.as_mut_slice(), E::C(0xab, 0xabcd)).unwrap();

    if let E::C(a, b) = e {
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
    let s = E::interpret(m.as_slice()).unwrap();

    if let E::C(a, b) = s {
        assert_eq!(*a, 0xab);
        assert_eq!(*b, 0xabcd);
    } else {
        panic!();
    }
}
