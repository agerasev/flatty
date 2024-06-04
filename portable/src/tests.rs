use crate::le;
use core::mem::align_of_val;
use flatty_base::traits::*;
use flatty_containers::vec::FlatVec;

#[test]
fn vec() {
    let mut bytes = [0u8; 2 + 3 * 4];
    let flat_vec = FlatVec::<le::I32, le::U16>::default_in_place(&mut bytes).unwrap();

    flat_vec.push(le::I32::from(0)).unwrap();
    flat_vec.push(le::I32::from(1)).unwrap();
    flat_vec.push(le::I32::from(2)).unwrap();
    assert!(flat_vec.push(le::I32::from(3)).is_err());

    assert_eq!(FlatVec::<le::I32, le::U16>::ALIGN, 1);
    assert_eq!(align_of_val(flat_vec), 1);
}
