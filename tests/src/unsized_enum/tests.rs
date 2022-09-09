macro_rules! generate_tests {
    () => {
        mod tests {
            use super::{UnsizedEnum, UnsizedEnumMut, UnsizedEnumRef, UnsizedEnumTag};
            use core::mem::{align_of_val, size_of_val};
            use flatty::{prelude::*, Error, ErrorKind};

            #[test]
            fn init_a() {
                let mut mem = vec![0u8; 2];
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                assert_eq!(ue.size(), 2);

                match ue.as_ref() {
                    UnsizedEnumRef::A => (),
                    _ => panic!(),
                }

                assert_eq!(mem[0], 0);
            }

            #[test]
            fn init_b() {
                let mut mem = vec![0u8; 6];
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                ue.set_default(UnsizedEnumTag::B).unwrap();
                if let UnsizedEnumMut::B(b0, b1) = ue.as_mut() {
                    *b0 = 0xab;
                    *b1 = 0xcdef;
                } else {
                    unreachable!();
                }
                assert_eq!(ue.size(), 6);

                match ue.as_ref() {
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
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                ue.set_default(UnsizedEnumTag::C).unwrap();
                if let UnsizedEnumMut::C { a, b } = ue.as_mut() {
                    *a = 0xab;
                    b.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]);
                } else {
                    unreachable!();
                }
                assert_eq!(ue.size(), 10);

                match ue.as_mut() {
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
                assert_eq!(ue.size(), 12);

                assert_eq!(mem[0], 2);
                assert_eq!(mem[2], 0xab);
                assert_eq!(&mem[4..6], [6, 0]);
                assert_eq!(&mem[6..], [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
            }

            #[test]
            fn tag() {
                let mut mem = vec![0u8; 6];
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                assert_eq!(ue.tag(), UnsizedEnumTag::A);
                ue.set_default(UnsizedEnumTag::B).unwrap();
                assert_eq!(ue.tag(), UnsizedEnumTag::B);
            }

            #[test]
            fn init_err() {
                let mut mem = vec![0u8; 1];
                let res = UnsizedEnum::placement_default(mem.as_mut_slice());
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InsufficientSize,
                        pos: 0
                    }
                );
            }

            #[test]
            fn set_err() {
                let mut mem = vec![0u8; 2];
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                assert_eq!(ue.tag(), UnsizedEnumTag::A);
                let res = ue.set_default(UnsizedEnumTag::B);
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InsufficientSize,
                        pos: 2
                    }
                );
            }

            #[test]
            fn bad_tag() {
                let mut mem = vec![4u8, 0u8];
                let res = UnsizedEnum::from_bytes(mem.as_mut_slice());
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InvalidEnumTag,
                        pos: 0
                    }
                );
            }

            #[test]
            fn layout() {
                let mut mem = vec![0u8; 6 + 8 * 2 + 1];
                let ue = UnsizedEnum::placement_default(mem.as_mut_slice()).unwrap();
                ue.set_default(UnsizedEnumTag::C).unwrap();
                if let UnsizedEnumMut::C { a, b } = ue.as_mut() {
                    *a = 0xab;
                    for i in 0.. {
                        if b.push(i).is_err() {
                            break;
                        }
                    }
                } else {
                    unreachable!();
                }

                assert_eq!(UnsizedEnum::DATA_OFFSET, 2);
                assert_eq!(align_of_val(ue), <UnsizedEnum as FlatBase>::ALIGN);
                assert_eq!(size_of_val(ue), ue.size());
                assert_eq!(ue.size(), mem.len() - 1);
            }
        }
    };
}

pub(crate) use generate_tests;
