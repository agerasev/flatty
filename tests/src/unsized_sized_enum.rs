use flatty::{error::ErrorKind, flat, prelude::*, utils::alloc::AlignedBytes};

#[flat(sized = false, default = true)]
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
    let mut mem = AlignedBytes::new(2, 2);
    let use_ = UnsizedSizedEnum::default_in_place(&mut mem).unwrap();
    assert_eq!(use_.size(), 2);

    match use_.as_ref() {
        UnsizedSizedEnumRef::A => (),
        _ => panic!(),
    }

    assert_eq!(mem[0], 0);
}

#[test]
fn init_b() {
    let mut mem = AlignedBytes::new(6, 2);
    let use_ = UnsizedSizedEnum::new_in_place(&mut mem, UnsizedSizedEnumInitB(0xab, 0xcdef)).unwrap();
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
    let mut mem = AlignedBytes::new(10, 2);
    let use_ = UnsizedSizedEnum::new_in_place(
        &mut mem,
        UnsizedSizedEnumInitC {
            a: 0xab,
            b: 0xcdef,
            c: [0x12, 0x34, 0x56, 0x78],
        },
    )
    .unwrap();
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
fn from_bytes_err() {
    let mut mem = AlignedBytes::new(1, 2);
    let res = UnsizedSizedEnum::from_mut_bytes(&mut mem);
    assert_eq!(res.err().unwrap().kind, ErrorKind::InsufficientSize);
}

#[test]
fn init_err() {
    let mut mem = AlignedBytes::new(3, 2);
    let res = UnsizedSizedEnum::new_in_place(&mut mem, UnsizedSizedEnumInitB(0, 0));
    assert_eq!(res.err().unwrap().kind, ErrorKind::InsufficientSize);
}
