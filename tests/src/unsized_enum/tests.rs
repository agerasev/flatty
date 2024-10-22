macro_rules! generate_tests {
    () => {
        mod tests {
            use super::{UnsizedEnum, UnsizedEnumInitB, UnsizedEnumInitC, UnsizedEnumMut, UnsizedEnumRef, UnsizedEnumTag};
            use core::mem::{align_of_val, size_of_val};
            use flatty::{
                error::{Error, ErrorKind},
                flat_vec,
                prelude::*,
                AlignedBytes,
            };

            #[test]
            fn init_a() {
                let mut mem = AlignedBytes::new(8, 4);
                let ue = UnsizedEnum::default_in_place(&mut mem).unwrap();
                assert_eq!(ue.size(), 4);

                match ue.as_ref() {
                    UnsizedEnumRef::A => (),
                    _ => panic!(),
                }

                assert_eq!(mem[0], 0);
            }

            #[test]
            fn init_b() {
                let mut mem = AlignedBytes::new(8, 4);
                let ue = UnsizedEnum::new_in_place(&mut mem, UnsizedEnumInitB(0xab, 0xcdef)).unwrap();
                assert_eq!(ue.size(), 8);

                match ue.as_ref() {
                    UnsizedEnumRef::B(x, y) => {
                        assert_eq!(*x, 0xab);
                        assert_eq!(*y, 0xcdef);
                    }
                    _ => panic!(),
                }

                assert_eq!(mem[0], 1);
                assert_eq!(mem[4], 0xab);
                assert_eq!(&mem[6..], [0xef, 0xcd]);
            }

            #[test]
            fn init_c() {
                let mut mem = AlignedBytes::new(16, 4);
                let ue = UnsizedEnum::new_in_place(
                    &mut mem,
                    UnsizedEnumInitC {
                        offset: 0x12345678,
                        bytes: flat_vec![0x12, 0x34, 0x56, 0x78],
                    },
                )
                .unwrap();
                assert_eq!(ue.size(), 16);

                match ue.as_mut() {
                    UnsizedEnumMut::C { offset, bytes } => {
                        assert_eq!(*offset, 0x12345678);
                        assert_eq!(bytes.len(), 4);
                        assert_eq!(bytes.capacity(), 6);
                        bytes.push(0x9a).unwrap();
                        bytes.push(0xbc).unwrap();
                        assert!(bytes.push(0xde).is_err());
                    }
                    _ => panic!(),
                }
                assert_eq!(ue.size(), 16);

                assert_eq!(mem[0], 2);
                assert_eq!(mem[4..8], [0x78, 0x56, 0x34, 0x12]);
                assert_eq!(&mem[8..10], [6, 0]);
                assert_eq!(&mem[10..], [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
            }

            #[test]
            fn tag() {
                let mut mem = AlignedBytes::new(8, 4);
                let ue = UnsizedEnum::default_in_place(&mut mem).unwrap();
                assert_eq!(ue.tag(), UnsizedEnumTag::A);
                ue.assign_in_place(UnsizedEnumInitB(0, 0)).unwrap();
                assert_eq!(ue.tag(), UnsizedEnumTag::B);
            }

            #[test]
            fn from_bytes_err() {
                let mut mem = AlignedBytes::new(1, 2);
                let res = UnsizedEnum::from_mut_bytes(&mut mem);
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InsufficientSize,
                        pos: 0
                    }
                );
            }

            #[test]
            fn validate_err() {
                let mut mem = AlignedBytes::from_slice(&[1, 0, 0, 0, 0, 0], 4);
                let res = UnsizedEnum::from_mut_bytes(&mut mem);
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InsufficientSize,
                        pos: 4
                    }
                );
            }

            #[test]
            fn set_err() {
                let mut mem = AlignedBytes::new(4, 4);
                let ue = UnsizedEnum::default_in_place(&mut mem).unwrap();
                let res = ue.assign_in_place(UnsizedEnumInitB(0, 0));
                assert_eq!(
                    res.err().unwrap(),
                    Error {
                        kind: ErrorKind::InsufficientSize,
                        pos: 4
                    }
                );
            }

            #[test]
            fn bad_tag() {
                let mem = AlignedBytes::from_slice(&[4, 0, 0, 0], 4);
                let res = UnsizedEnum::from_bytes(&mem);
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
                let mut mem = AlignedBytes::new(4 + 4 + 2 + 6 + 3, 2);
                let ue = UnsizedEnum::new_in_place(
                    &mut mem,
                    UnsizedEnumInitC {
                        offset: 0x12345678,
                        bytes: flat_vec![],
                    },
                )
                .unwrap();
                if let UnsizedEnumMut::C { offset: _, bytes } = ue.as_mut() {
                    for i in 0.. {
                        if bytes.push(i).is_err() {
                            break;
                        }
                    }
                } else {
                    unreachable!();
                }

                assert_eq!(UnsizedEnum::DATA_OFFSET, 4);
                assert_eq!(align_of_val(ue), <UnsizedEnum as FlatBase>::ALIGN);
                assert_eq!(size_of_val(ue), ue.size());
                assert_eq!(ue.size(), mem.len() - 3);
            }
        }
    };
}

pub(crate) use generate_tests;
