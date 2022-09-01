use core::mem::{align_of_val, size_of_val};
use flatty::{make_flat, FlatBase, FlatInit, FlatVec};

#[make_flat(sized = false)]
#[derive(Default, Debug, PartialEq, Eq)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64>,
}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let usd = UnsizedStructDyn {
        a: 200,
        b: 40000,
        c: vec![0, 1],
    };
    let us = UnsizedStruct::placement_new(mem.as_mut_slice(), &usd).unwrap();

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
    let mut mem = vec![0u8; 16 + 8 * 4];
    let us = UnsizedStruct::placement_default(mem.as_mut_slice()).unwrap();

    assert_eq!(us.size(), 16);
    assert_eq!(us.a, 0);
    assert_eq!(us.b, 0);
    assert_eq!(us.c.len(), 0);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let us = UnsizedStruct::placement_new(
        mem.as_mut_slice(),
        &UnsizedStructDyn {
            a: 0,
            b: 0,
            c: Vec::new(),
        },
    )
    .unwrap();
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
    let mut mem_ab = vec![0u8; 16 + 8 * 4];
    let mut mem_c = vec![0u8; 16 + 8 * 3];
    UnsizedStruct::placement_new(
        &mut mem_ab,
        &UnsizedStructDyn {
            a: 1,
            b: 2,
            c: vec![3, 4, 5, 6],
        },
    )
    .unwrap();
    let us_a = UnsizedStruct::reinterpret(&mem_ab).unwrap();
    let us_b = UnsizedStruct::reinterpret(&mem_ab).unwrap();
    let us_c = UnsizedStruct::placement_new(
        &mut mem_c,
        &UnsizedStructDyn {
            a: 1,
            b: 2,
            c: vec![3, 4, 5],
        },
    )
    .unwrap();

    assert_eq!(us_a, us_b);
    assert_ne!(us_a, us_c);
    assert_ne!(us_b, us_c);
}
