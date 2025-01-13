#[cfg(feature = "alloc")]
use crate::bytes::AlignedBytes;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, ffi::CString, rc::Rc, string::String, sync::Arc, vec::Vec};
use core::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
};
use flatty_base::{
    emplacer::Emplacer,
    error::Error,
    traits::{Flat, FlatDefault},
};
use stavec::{
    traits::{Container, Length},
    GenericVec,
};

/// Extra guarantees for [`AsRef<[u8]>`] and [`AsMut<[u8]>`].
///
/// # Safety
///
/// Types implementing this trait must guarantee that data referred by `as_ref` and `as_mut` (and reference metadata)
/// will not change internally between these calls (excluding interior mutability).
pub unsafe trait TrustedRef {}

/// Wrapper for smart pointer to byte slice that maps it to flat type.
pub struct FlatWrap<F: Flat + ?Sized, P: AsRef<[u8]> + TrustedRef> {
    pointer: P,
    _ghost: PhantomData<F>,
}

impl<F: Flat + ?Sized, P: AsRef<[u8]> + TrustedRef> FlatWrap<F, P> {
    /// # Safety
    ///
    /// Data behind pointer must be valid.
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
        F::validate(pointer.as_ref())?;
        Ok(unsafe { Self::from_wrapped_bytes_unchecked(pointer) })
    }
}

impl<F: Flat + ?Sized, P: AsRef<[u8]> + AsMut<[u8]> + TrustedRef> FlatWrap<F, P> {
    pub fn new_in_place(mut pointer: P, emplacer: impl Emplacer<F>) -> Result<Self, Error> {
        F::new_in_place(pointer.as_mut(), emplacer)?;
        Ok(unsafe { Self::from_wrapped_bytes_unchecked(pointer) })
    }

    pub fn default_in_place(pointer: P) -> Result<Self, Error>
    where
        F: FlatDefault,
    {
        Self::new_in_place(pointer, F::default_emplacer())
    }
}

impl<F: Flat + ?Sized, P: AsRef<[u8]> + TrustedRef> Deref for FlatWrap<F, P> {
    type Target = F;
    fn deref(&self) -> &Self::Target {
        unsafe { F::from_bytes_unchecked(self.pointer.as_ref()) }
    }
}
impl<F: Flat + ?Sized, P: AsRef<[u8]> + AsMut<[u8]> + TrustedRef> DerefMut for FlatWrap<F, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { F::from_mut_bytes_unchecked(self.pointer.as_mut()) }
    }
}

unsafe impl<T: ?Sized> TrustedRef for &T {}
unsafe impl<T: ?Sized> TrustedRef for &mut T {}

unsafe impl<P: Deref> TrustedRef for Pin<P> {}
unsafe impl<T: ?Sized> TrustedRef for ManuallyDrop<T> {}
unsafe impl<T: ?Sized> TrustedRef for Ref<'_, T> {}
unsafe impl<T: ?Sized> TrustedRef for RefMut<'_, T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedRef for Box<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedRef for Rc<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> TrustedRef for Arc<T> {}

#[cfg(feature = "alloc")]
unsafe impl<T> TrustedRef for Vec<T> {}
#[cfg(feature = "alloc")]
unsafe impl TrustedRef for String {}
#[cfg(feature = "alloc")]
unsafe impl TrustedRef for CString {}

unsafe impl<C: Container + ?Sized, L: Length> TrustedRef for GenericVec<C, L> {}
#[cfg(feature = "alloc")]
unsafe impl TrustedRef for AlignedBytes {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{bytes::AlignedBytes, vec::FlatVec};

    #[test]
    fn wrapped_vec() {
        let mut wrap = FlatWrap::<FlatVec<u8, u16>, _>::default_in_place(AlignedBytes::new(2 + 4, 2)).unwrap();

        assert_eq!(wrap.capacity(), 4);
        assert_eq!(wrap.len(), 0);
        assert_eq!(wrap.remaining(), 4);

        wrap.push_slice(&[1, 2]).unwrap();
        assert_eq!(wrap.len(), 2);
        assert_eq!(wrap.remaining(), 2);
        assert_eq!(wrap.as_slice(), [1, 2].as_slice());

        wrap.push_slice(&[3, 4]).unwrap();
        assert_eq!(wrap.len(), 4);
        assert_eq!(wrap.remaining(), 0);
        assert_eq!(wrap.as_slice(), [1, 2, 3, 4].as_slice());

        let bytes = wrap.into_inner();
        assert_eq!(bytes.deref(), [4, 0, 1, 2, 3, 4].as_slice())
    }
}
