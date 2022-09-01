use core::mem::{align_of, size_of};
use flatty::{mem::Muu, prelude::*, utils::ceil_mul, Error};

//#[make_flat]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[repr(C)]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}

impl FlatCast for SizedStruct {
    fn validate(this: &Muu<Self>) -> Result<(), Error> {
        let mut pos = 0;
        let bytes = this.as_bytes();

        u8::validate(unsafe { Muu::<u8>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u8::SIZE, u16::ALIGN);

        u16::validate(unsafe { Muu::<u16>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u16::SIZE, u32::ALIGN);

        u32::validate(unsafe { Muu::<u32>::from_bytes_unchecked(bytes.get_unchecked(pos..)) })
            .map_err(|e| e.offset(pos))?;
        pos += ceil_mul(pos + u32::SIZE, <[u64; 4]>::ALIGN);

        <[u64; 4]>::validate(unsafe {
            Muu::<[u64; 4]>::from_bytes_unchecked(bytes.get_unchecked(pos..))
        })
        .map_err(|e| e.offset(pos))?;

        Ok(())
    }
}

unsafe impl Flat for SizedStruct {}

#[test]
fn init() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let ss = SizedStruct::placement_default(m.as_mut_slice()).unwrap();
    ss.a = 200;
    ss.b = 40000;
    ss.c = 2000000000;
    ss.d = [1, 2, 3, 4];

    assert_eq!(ss.a, 200);
    assert_eq!(ss.b, 40000);
    assert_eq!(ss.c, 2000000000);
    assert_eq!(ss.d, [1, 2, 3, 4]);
}

#[test]
fn default() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let ss = SizedStruct::placement_default(m.as_mut_slice()).unwrap();

    assert_eq!(ss.a, u8::default());
    assert_eq!(ss.b, u16::default());
    assert_eq!(ss.c, u32::default());
    assert_eq!(ss.d, <[u64; 4]>::default());
}

#[test]
fn interpret() {
    let m = (0..4).fold(
        vec![0x12, 0xff, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12],
        |mut a, i| {
            a.extend([i + 1, 0, 0, 0, 0, 0, 0, 0].into_iter());
            a
        },
    );
    println!("{:?}", m.as_slice());
    let ss = SizedStruct::from_bytes(m.as_slice()).unwrap();

    assert_eq!(ss.a, 0x12);
    assert_eq!(ss.b, 0x1234);
    assert_eq!(ss.c, 0x12345678);
    assert_eq!(ss.d, [1, 2, 3, 4]);
}

#[test]
fn layout() {
    let mut m = vec![0u8; 16 + 8 * 4];
    let ss = SizedStruct::placement_default(m.as_mut_slice()).unwrap();

    assert_eq!(align_of::<SizedStruct>(), <SizedStruct as FlatBase>::ALIGN);
    assert_eq!(size_of::<SizedStruct>(), <SizedStruct as FlatSized>::SIZE);
    assert_eq!(<SizedStruct as FlatSized>::SIZE, ss.size());
}
