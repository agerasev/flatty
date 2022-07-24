use core::mem::{align_of, size_of};
use flatty::{make_flat, FlatBase, FlatInit, FlatSized};

#[make_flat(enum_type = "u8")]
#[derive(Default)]
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

#[test]
fn init_a() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::init_default(m.as_mut_slice()).unwrap();

    if let SizedEnum::A = se {
    } else {
        panic!();
    }

    assert_eq!(m[0], 0);
}

#[test]
fn init_b() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::init(m.as_mut_slice(), SizedEnum::B(0x1234, 0x56)).unwrap();

    if let SizedEnum::B(a, b) = se {
        assert_eq!(*a, 0x1234);
        assert_eq!(*b, 0x56);
    } else {
        panic!();
    }

    assert_eq!(m[0], 1);
    assert_eq!(&m[4..7], [0x34, 0x12, 0x56]);
}

#[test]
fn init_c() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::init(m.as_mut_slice(), SizedEnum::C { a: 0xab, b: 0xcdef }).unwrap();

    if let SizedEnum::C { a, b } = se {
        assert_eq!(*a, 0xab);
        assert_eq!(*b, 0xcdef);
    } else {
        panic!();
    }

    assert_eq!(m[0], 2);
    assert_eq!(m[4], 0xab);
    assert_eq!(&m[6..], [0xef, 0xcd]);
}

#[test]
fn init_d() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::init(m.as_mut_slice(), SizedEnum::D(0x12345678)).unwrap();

    if let SizedEnum::D(a) = se {
        assert_eq!(*a, 0x12345678);
    } else {
        panic!();
    }

    assert_eq!(m[0], 3);
    assert_eq!(&m[4..], [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn interpret_c() {
    let m = vec![2, 0, 0, 0, 0xab, 0, 0xef, 0xcd];
    let s = SizedEnum::interpret(m.as_slice()).unwrap();

    if let SizedEnum::C { a, b } = s {
        assert_eq!(*a, 0xab);
        assert_eq!(*b, 0xcdef);
    } else {
        panic!();
    }
}

#[test]
fn layout() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::init_default(m.as_mut_slice()).unwrap();

    assert_eq!(align_of::<SizedEnum>(), <SizedEnum as FlatBase>::ALIGN);
    assert_eq!(size_of::<SizedEnum>(), <SizedEnum as FlatSized>::SIZE);
    assert_eq!(<SizedEnum as FlatSized>::SIZE, se.size());
}
