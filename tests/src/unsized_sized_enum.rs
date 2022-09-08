use flatty::{make_flat, prelude::*, ErrorKind};

#[make_flat(sized = false, enum_type = "u8")]
#[derive(Default, Debug, PartialEq, Eq)]
enum UnsizedSizedEnum {
    #[default]
    A,
    B(u8, u16),
    C {
        a: u8,
        b: u16,
        c: [u8; 4],
    },
}

#[test]
fn init_a() {
    let mut mem = vec![0u8; 2];
    let use_ = UnsizedSizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    assert_eq!(use_.size(), 2);

    match use_.as_ref() {
        UnsizedSizedEnumRef::A => (),
        _ => panic!(),
    }

    assert_eq!(mem[0], 0);
}

#[test]
fn init_b() {
    let mut mem = vec![0u8; 6];
    let use_ = UnsizedSizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    use_.set_default(UnsizedSizedEnumTag::B).unwrap();
    if let UnsizedSizedEnumMut::B(b0, b1) = use_.as_mut() {
        *b0 = 0xab;
        *b1 = 0xcdef;
    } else {
        unreachable!();
    }
    assert_eq!(use_.size(), 6);

    match use_.as_ref() {
        UnsizedSizedEnumRef::B(x, y) => {
            assert_eq!(*x, 0xab);
            assert_eq!(*y, 0xcdef);
        }
        _ => panic!(),
    }

    assert_eq!(mem[0], 1);
    assert_eq!(mem[2], 0xab);
    assert_eq!(&mem[4..], [0xef, 0xcd]);
}

#[test]
fn init_c() {
    let mut mem = vec![0u8; 10];
    let use_ = UnsizedSizedEnum::placement_default(mem.as_mut_slice()).unwrap();
    use_.set_default(UnsizedSizedEnumTag::C).unwrap();
    if let UnsizedSizedEnumMut::C { a, b, c } = use_.as_mut() {
        *a = 0xab;
        *b = 0xcdef;
        *c = [0x12, 0x34, 0x56, 0x78];
    } else {
        unreachable!();
    }
    assert_eq!(use_.size(), 10);

    match use_.as_mut() {
        UnsizedSizedEnumMut::C { a, b, c } => {
            assert_eq!(*a, 0xab);
            assert_eq!(*b, 0xcdef);
            assert_eq!(*c, [0x12, 0x34, 0x56, 0x78]);
        }
        _ => panic!(),
    }

    assert_eq!(mem[0], 2);
    assert_eq!(mem[2], 0xab);
    assert_eq!(&mem[4..6], [0xef, 0xcd]);
    assert_eq!(&mem[6..], [0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn init_err() {
    let mut mem = vec![0u8; 1];
    let res = UnsizedSizedEnum::placement_default(mem.as_mut_slice());
    assert_eq!(res.err().unwrap().kind, ErrorKind::InsufficientSize);
}
