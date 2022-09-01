use crate::le;
use base::{prelude::*, FlatVec};
use core::mem::align_of_val;

#[test]
fn vec() {
    let mut bytes = vec![0u8; 2 + 3 * 4];
    let flat_vec = FlatVec::<le::I32, le::U16>::placement_default(bytes.as_mut_slice()).unwrap();

    flat_vec.push(le::I32::from(0)).unwrap();
    flat_vec.push(le::I32::from(1)).unwrap();
    flat_vec.push(le::I32::from(2)).unwrap();
    assert!(flat_vec.push(le::I32::from(3)).is_err());

    assert_eq!(FlatVec::<le::I32, le::U16>::ALIGN, 1);
    assert_eq!(align_of_val(flat_vec), 1);
}
