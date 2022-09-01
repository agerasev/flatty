use core::mem::{align_of, size_of};
use flatty::{mem::Muu, prelude::*, Error, ErrorKind};

//#[make_flat(enum_type = "u8")]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[repr(C, u8)]
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

impl FlatCast for SizedEnum {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let index = unsafe { Muu::<u8>::from_bytes_unchecked(this.as_bytes().get_unchecked(0..)) };
        u8::validate(index)?;
        match unsafe { &*index.as_ptr() } {
            0 => Ok(()),
            1 => {
                u16::validate(unsafe {
                    Muu::<u16>::from_bytes_unchecked(this.as_bytes().get_unchecked(4..))
                })
                .map_err(|e| e.offset(4))?;

                u8::validate(unsafe {
                    Muu::<u8>::from_bytes_unchecked(this.as_bytes().get_unchecked(6..))
                })
                .map_err(|e| e.offset(6))?;

                Ok(())
            }
            2 => {
                u8::validate(unsafe {
                    Muu::<u8>::from_bytes_unchecked(this.as_bytes().get_unchecked(4..))
                })
                .map_err(|e| e.offset(4))?;

                u16::validate(unsafe {
                    Muu::<u16>::from_bytes_unchecked(this.as_bytes().get_unchecked(6..))
                })
                .map_err(|e| e.offset(6))?;

                Ok(())
            }
            3 => {
                u32::validate(unsafe {
                    Muu::<u32>::from_bytes_unchecked(this.as_bytes().get_unchecked(4..))
                })
                .map_err(|e| e.offset(4))?;

                Ok(())
            }
            _ => Err(Error {
                kind: ErrorKind::InvalidEnumState,
                pos: 0,
            }),
        }
    }
}

unsafe impl Flat for SizedEnum {}

#[test]
fn init_a() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::placement_default(m.as_mut_slice()).unwrap();

    if let SizedEnum::A = se {
    } else {
        panic!();
    }

    assert_eq!(m[0], 0);
}

#[test]
fn init_b() {
    let mut m = vec![0u8; 4 + 4];
    let se = SizedEnum::placement_new(m.as_mut_slice(), &SizedEnum::B(0x1234, 0x56)).unwrap();

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
    let se =
        SizedEnum::placement_new(m.as_mut_slice(), &SizedEnum::C { a: 0xab, b: 0xcdef }).unwrap();

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
    let se = SizedEnum::placement_new(m.as_mut_slice(), &SizedEnum::D(0x12345678)).unwrap();

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
    let s = SizedEnum::reinterpret(m.as_slice()).unwrap();

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
    let se = SizedEnum::placement_default(m.as_mut_slice()).unwrap();

    assert_eq!(align_of::<SizedEnum>(), <SizedEnum as FlatBase>::ALIGN);
    assert_eq!(size_of::<SizedEnum>(), <SizedEnum as FlatSized>::SIZE);
    assert_eq!(<SizedEnum as FlatSized>::SIZE, se.size());
}
