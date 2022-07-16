use crate::FlatSized;

unsafe impl FlatSized for () {}

unsafe impl FlatSized for bool {}

unsafe impl FlatSized for u8 {}
unsafe impl FlatSized for u16 {}
unsafe impl FlatSized for u32 {}
unsafe impl FlatSized for u64 {}
unsafe impl FlatSized for u128 {}

unsafe impl FlatSized for i8 {}
unsafe impl FlatSized for i16 {}
unsafe impl FlatSized for i32 {}
unsafe impl FlatSized for i64 {}
unsafe impl FlatSized for i128 {}

unsafe impl FlatSized for f32 {}
unsafe impl FlatSized for f64 {}

unsafe impl<T: FlatSized + Sized, const N: usize> FlatSized for [T; N] {}
