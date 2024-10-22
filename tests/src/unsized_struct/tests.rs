macro_rules! generate_tests {
    () => {
        mod tests {
            use super::{UnsizedStruct, UnsizedStructInit};
            use core::mem::{align_of_val, size_of_val};
            use flatty::{flat_vec, prelude::*, AlignedBytes};

            #[test]
            fn init() {
                let mut mem = AlignedBytes::new(16 + 4 * 8, 8);
                let us = UnsizedStruct::new_in_place(
                    &mut mem,
                    UnsizedStructInit {
                        a: 200,
                        b: 40000,
                        c: flat_vec![0, 1],
                    },
                )
                .unwrap();

                assert_eq!(us.size(), 32);
                assert_eq!(us.a, 200);
                assert_eq!(us.b, 40000);
                assert_eq!(us.c.len(), 2);

                for i in 2.. {
                    if us.c.push(i).is_err() {
                        break;
                    }
                }

                assert_eq!(us.size(), 48);
                assert_eq!(us.a, 200);
                assert_eq!(us.b, 40000);
                assert_eq!(us.c.len(), 4);
                assert_eq!(us.c.as_slice(), [0, 1, 2, 3]);
            }

            #[test]
            fn default() {
                let mut mem = AlignedBytes::new(16 + 4 * 8, 8);
                let us = UnsizedStruct::default_in_place(&mut mem).unwrap();

                assert_eq!(us.size(), 16);
                assert_eq!(us.a, 0);
                assert_eq!(us.b, 0);
                assert_eq!(us.c.len(), 0);
            }

            #[test]
            fn layout() {
                let mut mem = AlignedBytes::new(16 + 4 * 8, 8);
                let us = UnsizedStruct::default_in_place(&mut mem).unwrap();
                us.a = 0;
                us.b = 0;
                for i in 0.. {
                    if us.c.push(i).is_err() {
                        break;
                    }
                }

                assert_eq!(align_of_val(us), <UnsizedStruct as FlatBase>::ALIGN);
                assert_eq!(size_of_val(us), us.size());
                assert_eq!(us.size(), mem.len());
            }

            #[test]
            fn eq() {
                let mut mem_ab = AlignedBytes::new(16 + 4 * 8, 8);
                let mut mem_c = AlignedBytes::new(16 + 3 * 8, 8);
                {
                    UnsizedStruct::new_in_place(
                        &mut mem_ab,
                        UnsizedStructInit {
                            a: 1,
                            b: 2,
                            c: flat_vec![3, 4, 5, 6],
                        },
                    )
                    .unwrap();
                }
                let us_a = UnsizedStruct::from_bytes(&mem_ab).unwrap();
                let us_b = UnsizedStruct::from_bytes(&mem_ab).unwrap();
                let us_c = UnsizedStruct::new_in_place(
                    &mut mem_c,
                    UnsizedStructInit {
                        a: 1,
                        b: 2,
                        c: flat_vec![3, 4, 5],
                    },
                )
                .unwrap();

                assert_eq!(us_a, us_b);
                assert_ne!(us_a, us_c);
                assert_ne!(us_b, us_c);
            }
        }
    };
}

pub(crate) use generate_tests;
