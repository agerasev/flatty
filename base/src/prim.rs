use crate::{mem::Unvalidated, Error, Flat, FlatValidate};

/// Macro for implementing [`Flat`] for primitive types.
///
/// # Safety
///
/// `$ty` must be [`Sized`]` + `[`Copy`].
///
/// Any possible memory state of the variable of the type must be valid.
macro_rules! impl_flat_prim {
    ($ty:ty) => {
        impl FlatValidate for $ty {
            fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error> {
                unsafe { Ok(this.assume_init()) }
            }
        }

        unsafe impl Flat for $ty {}
    };
}

impl_flat_prim!(());

impl_flat_prim!(u8);
impl_flat_prim!(u16);
impl_flat_prim!(u32);
impl_flat_prim!(u64);
impl_flat_prim!(u128);
impl_flat_prim!(usize);

impl_flat_prim!(i8);
impl_flat_prim!(i16);
impl_flat_prim!(i32);
impl_flat_prim!(i64);
impl_flat_prim!(i128);
impl_flat_prim!(isize);

impl_flat_prim!(f32);
impl_flat_prim!(f64);
