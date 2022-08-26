use crate::{Error, Flat, FlatInit};
use core::mem;

/// Macro for implementing [`Flat`] for primitive types..
///
/// # Safety
///
/// Type must be [`Sized`]` + `[`Copy`].
///
/// Any possible memory state of the variable of the type must be valid.
macro_rules! impl_flat_prim {
    ($ty:ty) => {
        impl FlatInit for $ty {
            type Dyn = $ty;

            fn size_of(_: &$ty) -> usize {
                mem::size_of::<$ty>()
            }

            unsafe fn placement_new_unchecked<'a, 'b>(
                mem: &'a mut [u8],
                init: &'b $ty,
            ) -> &'a mut Self {
                let self_ = Self::reinterpret_mut_unchecked(mem);
                *self_ = *init;
                self_
            }

            fn pre_validate(_: &[u8]) -> Result<(), Error> {
                Ok(())
            }
            fn post_validate(&self) -> Result<(), Error> {
                Ok(())
            }

            unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
                &*(mem.as_ptr() as *const Self)
            }
            unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                &mut *(mem.as_mut_ptr() as *mut Self)
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
