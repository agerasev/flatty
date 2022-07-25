use flatty::{make_flat, FlatInit, FlatVec, InterpretError};

#[make_flat(sized = false, enum_type = "u8")]
enum UnsizedEnum {
    A,
    B(u8, u16),
    C { a: u8, b: FlatVec<u8, u16> },
}

#[test]
fn init_a() {
    let mut mem = vec![0u8; 2];
    let unsized_enum = UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::A).unwrap();

    match unsized_enum.as_ref() {
        UnsizedEnumRef::A => (),
        _ => panic!(),
    }

    assert_eq!(mem[0], 0);
}

#[test]
fn init_b() {
    let mut mem = vec![0u8; 6];
    let unsized_enum =
        UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::B(0xab, 0xcdef)).unwrap();

    match unsized_enum.as_ref() {
        UnsizedEnumRef::B(x, y) => {
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
    let mut mem = vec![0u8; 12];
    let unsized_enum = UnsizedEnum::init(
        mem.as_mut_slice(),
        UnsizedEnumInit::C {
            a: 0xab,
            b: vec![0x12, 0x34, 0x56, 0x78],
        },
    )
    .unwrap();

    match unsized_enum.as_mut() {
        UnsizedEnumMut::C { a, b } => {
            assert_eq!(*a, 0xab);
            assert_eq!(b.len(), 4);
            assert_eq!(b.capacity(), 6);
            b.push(0x9a).unwrap();
            b.push(0xbc).unwrap();
            assert!(b.push(0xde).is_err());
        }
        _ => panic!(),
    }

    assert_eq!(mem[0], 2);
    assert_eq!(mem[2], 0xab);
    assert_eq!(&mem[4..6], [6, 0]);
    assert_eq!(&mem[6..], [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
}

#[test]
fn init_err() {
    let mut mem = vec![0u8; 1];
    let res = UnsizedEnum::init(mem.as_mut_slice(), UnsizedEnumInit::A);
    assert_eq!(res.err().unwrap(), InterpretError::InsufficientSize);
}
