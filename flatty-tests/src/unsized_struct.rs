use core::mem::{align_of_val, size_of_val};
use flatty::{make_flat, FlatBase, FlatInit, FlatVec};

#[make_flat(sized = false)]
//#[derive(Default)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64>,
}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let us = UnsizedStruct::placement_new(
        mem.as_mut_slice(),
        UnsizedStructInit {
            a: 200,
            b: 40000,
            c: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(us.a, 200);
    assert_eq!(us.b, 40000);
    assert_eq!(us.c.len(), 0);

    for i in 0.. {
        if us.c.push(i).is_err() {
            break;
        }
    }

    assert_eq!(us.a, 200);
    assert_eq!(us.b, 40000);
    assert_eq!(us.c.len(), 4);
    assert_eq!(us.c.as_slice(), [0, 1, 2, 3]);
}

#[test]
fn layout() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let us = UnsizedStruct::placement_new(
        mem.as_mut_slice(),
        UnsizedStructInit {
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
