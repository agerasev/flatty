use flatty::{make_flat, FlatInit, FlatUnsized, FlatVec};

#[make_flat(sized = false)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: FlatVec<u64>,
}

#[test]
fn init() {
    let mut mem = vec![0u8; 16 + 8 * 4];
    let unsized_struct = UnsizedStruct::init(
        mem.as_mut_slice(),
        UnsizedStructInit {
            a: 200,
            b: 40000,
            c: <FlatVec<u64> as FlatInit>::Init::Empty,
        },
    )
    .unwrap();

    assert_eq!(unsized_struct.a, 200);
    assert_eq!(unsized_struct.b, 40000);
    assert_eq!(unsized_struct.c.len(), 0);

    for i in 0.. {
        if unsized_struct.c.push(i).is_err() {
            break;
        }
    }

    assert_eq!(unsized_struct.a, 200);
    assert_eq!(unsized_struct.b, 40000);
    assert_eq!(unsized_struct.c.len(), 4);
    assert_eq!(unsized_struct.c.as_slice(), [0, 1, 2, 3]);
}
