use crate::{emplacer::Emplacer, error::Error, mem::MaybeUninitUnsized};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// Basic flat type preoperties.
pub unsafe trait FlatBase {
    /// Align of the type.
    const ALIGN: usize;
    /// Minimal size of an instance of the type.
    const MIN_SIZE: usize;

    /// Size of an instance of the type.
    fn size(&self) -> usize;
}

/// Dynamically-sized flat type. Like `?Sized` but for `Flat`.
///
/// For now it is implemented for all [`Flat`](`crate::Flat`) types because there is no mutually exclusive traits in Rust yet.
pub unsafe trait FlatUnsized: FlatBase {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;

    /// Metadata to store in a wide pointer to `Self`.
    ///
    /// Provides such value, that will give [`size()`](`FlatBase::size`)` <= `[`size_of_val()`](`core::mem::size_of_val`)` <= bytes.len()`.
    ///
    /// Panics if type is `Sized`.
    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize;
    /// Length of underlying memory occupied by `this`.
    fn bytes_len(this: &Self) -> usize;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn new_in_place<I: Emplacer<Self>>(this: &mut MaybeUninitUnsized<Self>, emplacer: I) -> Result<&mut Self, Error> {
        emplacer.emplace(this)
    }
    fn assign_in_place<I: Emplacer<Self>>(&mut self, emplacer: I) -> Result<&mut Self, Error> {
        Self::new_in_place(unsafe { self.as_mut_uninit() }, emplacer)
    }

    unsafe fn from_uninit_unchecked(this: &MaybeUninitUnsized<Self>) -> &Self;
    unsafe fn from_mut_uninit_unchecked(this: &mut MaybeUninitUnsized<Self>) -> &mut Self;

    fn as_uninit(&self) -> &MaybeUninitUnsized<Self>;
    /// # Safety
    ///
    /// Modification of return value must not make `self` invalid.
    unsafe fn as_mut_uninit(&mut self) -> &mut MaybeUninitUnsized<Self>;

    fn uninit_from_bytes(bytes: &[u8]) -> Result<&MaybeUninitUnsized<Self>, Error> {
        MaybeUninitUnsized::from_bytes(bytes)
    }
    fn uninit_from_mut_bytes(bytes: &mut [u8]) -> Result<&mut MaybeUninitUnsized<Self>, Error> {
        MaybeUninitUnsized::from_mut_bytes(bytes)
    }
}

#[macro_export]
macro_rules! impl_unsized_uninit_cast {
    () => {
        unsafe fn from_uninit_unchecked(this: &$crate::mem::MaybeUninitUnsized<Self>) -> &Self {
            let slice = ::core::ptr::slice_from_raw_parts(this.as_bytes().as_ptr(), Self::ptr_metadata(this));
            &*(slice as *const [_] as *const Self)
        }
        unsafe fn from_mut_uninit_unchecked(this: &mut $crate::mem::MaybeUninitUnsized<Self>) -> &mut Self {
            let slice = ::core::ptr::slice_from_raw_parts_mut(this.as_mut_bytes().as_mut_ptr(), Self::ptr_metadata(this));
            &mut *(slice as *mut [_] as *mut Self)
        }

        fn as_uninit(&self) -> &$crate::mem::MaybeUninitUnsized<Self> {
            unsafe {
                $crate::mem::MaybeUninitUnsized::from_bytes_unchecked(::core::slice::from_raw_parts(
                    self as *const _ as *const u8,
                    Self::bytes_len(self),
                ))
            }
        }
        unsafe fn as_mut_uninit(&mut self) -> &mut $crate::mem::MaybeUninitUnsized<Self> {
            $crate::mem::MaybeUninitUnsized::from_mut_bytes_unchecked(::core::slice::from_raw_parts_mut(
                self as *mut _ as *mut u8,
                Self::bytes_len(self),
            ))
        }
    };
}

/// Flat type runtime checking.
pub trait FlatCheck: FlatUnsized {
    /// FlatCheck that `this` is valid.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn validate(this: &MaybeUninitUnsized<Self>) -> Result<&Self, Error>;
    /// The same as [`Self::validate`] but returns mutable reference.
    fn validate_mut(this: &mut MaybeUninitUnsized<Self>) -> Result<&mut Self, Error> {
        Self::validate(this)?;
        unsafe { Ok(this.assume_init_mut()) }
    }

    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        Self::uninit_from_bytes(bytes)?.validate()
    }
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Self::uninit_from_mut_bytes(bytes)?.validate_mut()
    }
}

/// Flat type.
///
/// *If you want to implement this type for your custom type it's recommended to use safe `flat` macro instead.*
///
/// # Safety
///
/// By implementing this trait by yourself you guarantee:
///
/// + `Self` has stable binary representation that will not change in future.
///   (But the representation could be differ across different platforms. If you need such a guarantee see [`Portable`](`crate::Portable`).)
/// + `Self` don't own any resources outside of it.
/// + `Self` could be trivially copied as bytes. (We cannot require `Self: `[`Copy`] because it `?Sized`.)
/// + All methods of dependent traits have proper implemetation and will not cause an Undefined Behaviour.
pub unsafe trait Flat: FlatBase + FlatUnsized + FlatCheck {}
