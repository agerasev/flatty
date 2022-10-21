macro_rules! generate_tests {
    () => {
        mod tests {
            use super::SizedStruct;
            use core::mem::{align_of, size_of};
            use flatty::prelude::*;

            #[test]
            fn init() {
                let mut m = vec![0u8; 16 + 8 * 4];
                let ss = SizedStruct::from_mut_bytes(&mut m)
                    .unwrap()
                    .new_in_place(SizedStruct {
                        a: 200,
                        b: 40000,
                        c: 2000000000,
                        d: [1, 2, 3, 4],
                    })
                    .unwrap();

                assert_eq!(ss.a, 200);
                assert_eq!(ss.b, 40000);
                assert_eq!(ss.c, 2000000000);
                assert_eq!(ss.d, [1, 2, 3, 4]);
            }

            #[test]
            fn default() {
                let mut m = vec![0u8; 16 + 8 * 4];
                let ss = SizedStruct::from_mut_bytes(&mut m).unwrap().default_in_place().unwrap();

                assert_eq!(ss.a, u8::default());
                assert_eq!(ss.b, u16::default());
                assert_eq!(ss.c, u32::default());
                assert_eq!(ss.d, <[u64; 4]>::default());
            }

            #[test]
            fn interpret() {
                let m = (0..4).fold(vec![0x12, 0xff, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12], |mut a, i| {
                    a.extend([i + 1, 0, 0, 0, 0, 0, 0, 0].into_iter());
                    a
                });
                let ss = SizedStruct::from_bytes(&m).unwrap().validate().unwrap();

                assert_eq!(ss.a, 0x12);
                assert_eq!(ss.b, 0x1234);
                assert_eq!(ss.c, 0x12345678);
                assert_eq!(ss.d, [1, 2, 3, 4]);
            }

            #[test]
            fn layout() {
                let mut m = vec![0u8; 16 + 8 * 4];
                let ss = SizedStruct::from_mut_bytes(&mut m).unwrap().default_in_place().unwrap();

                assert_eq!(align_of::<SizedStruct>(), <SizedStruct as FlatBase>::ALIGN);
                assert_eq!(size_of::<SizedStruct>(), <SizedStruct as FlatSized>::SIZE);
                assert_eq!(<SizedStruct as FlatSized>::SIZE, ss.size());
            }
        }
    };
}

pub(crate) use generate_tests;
