#[cfg(feature = "alloc")]
use crate::utils::alloc::AlignedBytes;
use crate::{
    emplacer::Emplacer,
    error::Error,
    traits::{Flat, FlatDefault},
};
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, ffi::CString, rc::Rc, string::String, sync::Arc, vec::Vec};
use core::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
};
use stavec::{
    traits::{Container, Length},
    GenericVec,
};

/// Extra guarantees for [`Deref`] and [`DerefMut`].
///
/// # Safety
///
/// Types implementing this trait must guarantee that data referred by `deref` and `deref_mut` (and reference metadata)
/// will not change internally between these calls (excluding interior mutability).
pub unsafe trait TrustedDeref: Deref {}

/// Wrapper for smart pointer to byte slice that maps it to flat type.
pub struct FlatWrap<F: Flat + ?Sized, P: TrustedDeref<Target = [u8]>> {
    pointer: P,
    _ghost: PhantomData<F>,
}

impl<F: Flat + ?Sized, P: TrustedDeref<Target = [u8]>> FlatWrap<F, P> {
    pub unsafe fn from_wrapped_bytes_unchecked(pointer: P) -> Self {
        Self {
            pointer,
            _ghost: PhantomData,
        }
    }
    pub fn into_inner(self) -> P {
        self.pointer
    }

    pub fn from_wrapped_bytes(pointer: P) -> Result<Self, Error> {
        F::validate(&pointer)?;
        Ok(unsafe { Self::from_wrapped_bytes_unchecked(pointer) })
    }

    pub fn new_in_place(mut pointer: P, emplacer: impl Emplacer<F>) -> Result<Self, Error>
    where
        P: DerefMut,
    {
        F::new_in_place(&mut pointer, emplacer)?;
        Ok(unsafe { Self::from_wrapped_bytes_unchecked(pointer) })
    }
    pub fn default_in_place(pointer: P) -> Result<Self, Error>
    where
        P: DerefMut,
        F: FlatDefault,
    {
        Self::new_in_place(pointer, F::default_emplacer())
    }
}

impl<F: Flat + ?Sized, P: TrustedDeref<Target = [u8]>> Deref for FlatWrap<F, P> {
    type Target = F;
    fn deref(&self) -> &Self::Target {
        unsafe { F::from_bytes_unchecked(&self.pointer) }
    }
}
impl<F: Flat + ?Sized, P: TrustedDeref<Target = [u8]> + DerefMut> DerefMut for FlatWrap<F, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { F::from_mut_bytes_unchecked(&mut self.pointer) }
    }
}

unsafe impl<'a, T: ?Sized> TrustedDeref for &'a T {}
unsafe impl<'a, T: ?Sized> TrustedDeref for &'a mut T {}

unsafe impl<P: Deref> TrustedDeref for Pin<P> {}
unsafe impl<T: ?Sized> TrustedDeref for ManuallyDrop<T> {}
unsafe impl<'a, T: ?Sized> TrustedDeref for Ref<'a, T> {}
unsafe impl<'a, T: ?Sized> TrustedDeref for RefMut<'a, T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedDeref for Box<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedDeref for Rc<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedDeref for Arc<T> {}

#[cfg(feature = "alloc")]
unsafe impl<T> TrustedDeref for Vec<T> {}
#[cfg(feature = "alloc")]
unsafe impl TrustedDeref for String {}
#[cfg(feature = "alloc")]
unsafe impl TrustedDeref for CString {}

unsafe impl<C: Container + ?Sized, L: Length> TrustedDeref for GenericVec<C, L> {}
#[cfg(feature = "alloc")]
unsafe impl TrustedDeref for AlignedBytes {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{utils::alloc::AlignedBytes, vec::FlatVec};

    #[test]
    fn wrapped_vec() {
        let mut wrap = FlatWrap::<FlatVec<u8, u16>, _>::default_in_place(AlignedBytes::new(2 + 4, 2)).unwrap();

        assert_eq!(wrap.capacity(), 4);
        assert_eq!(wrap.len(), 0);
        assert_eq!(wrap.remaining(), 4);

        assert_eq!(wrap.extend_from_slice(&[1, 2]), 2);
        assert_eq!(wrap.len(), 2);
        assert_eq!(wrap.remaining(), 2);
        assert_eq!(wrap.as_slice(), [1, 2].as_slice());

        assert_eq!(wrap.extend_from_slice(&[3, 4, 5]), 2);
        assert_eq!(wrap.len(), 4);
        assert_eq!(wrap.remaining(), 0);
        assert_eq!(wrap.as_slice(), [1, 2, 3, 4].as_slice());

        let bytes = wrap.into_inner();
        assert_eq!(bytes.deref(), [4, 0, 1, 2, 3, 4].as_slice())
    }
}
