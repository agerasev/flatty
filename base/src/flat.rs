use crate::{
    emplacer::Emplacer,
    error::Error,
    mem::{AsBytes, Unvalidated},
};

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
/// *For now has to be implemented for all [`Flat`](`crate::Flat`) types because there is no mutually exclusive traits in Rust yet.*
pub unsafe trait FlatUnsized: FlatBase {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;
    /// Type to store unvalidated binary representation of `Self`.
    type AsBytes: AsBytes + ?Sized;

    /// Metadata to store in a wide pointer to `Self`.
    ///
    /// Panics if type is `Sized`.
    fn ptr_metadata(this: &Unvalidated<Self>) -> usize;
    /// Length of underlying memory occupied by `this`.
    fn bytes_len(this: *const Self) -> usize;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn new_in_place<I: Emplacer<Self>>(this: &mut Unvalidated<Self>, emplacer: I) -> Result<&mut Self, Error> {
        emplacer.emplace(this)
    }
    fn assign_in_place<I: Emplacer<Self>>(&mut self, emplacer: I) -> Result<&mut Self, Error> {
        Self::new_in_place(unsafe { self.as_mut_uninit() }, emplacer)
    }

    fn ptr_from_uninit(this: &Unvalidated<Self>) -> *const Self;
    fn ptr_from_mut_uninit(this: &mut Unvalidated<Self>) -> *mut Self;

    unsafe fn ptr_as_uninit<'a>(this: *const Self) -> &'a Unvalidated<Self>;
    unsafe fn ptr_as_mut_uninit<'a>(this: *mut Self) -> &'a mut Unvalidated<Self>;

    fn as_uninit(&self) -> &Unvalidated<Self> {
        unsafe { Self::ptr_as_uninit(self as *const _) }
    }
    /// # Safety
    ///
    /// Modification of return value must not make `self` invalid.
    unsafe fn as_mut_uninit(&mut self) -> &mut Unvalidated<Self> {
        Self::ptr_as_mut_uninit(self as *mut _)
    }

    fn from_bytes(bytes: &[u8]) -> Result<&Unvalidated<Self>, Error> {
        Unvalidated::from_bytes(bytes)
    }
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Unvalidated<Self>, Error> {
        Unvalidated::from_mut_bytes(bytes)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_unsized_uninit_cast {
    () => {
        type AsBytes = [u8];

        fn ptr_from_uninit(this: &$crate::mem::Unvalidated<Self>) -> *const Self {
            let slice = ::core::ptr::slice_from_raw_parts(this.as_bytes().as_ptr(), Self::ptr_metadata(this));
            slice as *const [_] as *const Self
        }
        fn ptr_from_mut_uninit(this: &mut $crate::mem::Unvalidated<Self>) -> *mut Self {
            let slice = ::core::ptr::slice_from_raw_parts_mut(this.as_mut_bytes().as_mut_ptr(), Self::ptr_metadata(this));
            slice as *mut [_] as *mut Self
        }

        unsafe fn ptr_as_uninit<'__uninit_a>(this: *const Self) -> &'__uninit_a $crate::mem::Unvalidated<Self> {
            unsafe {
                $crate::mem::Unvalidated::from_bytes_unchecked(::core::slice::from_raw_parts(
                    this as *const u8,
                    Self::bytes_len(this),
                ))
            }
        }
        unsafe fn ptr_as_mut_uninit<'__uninit_a>(this: *mut Self) -> &'__uninit_a mut $crate::mem::Unvalidated<Self> {
            $crate::mem::Unvalidated::from_mut_bytes_unchecked(::core::slice::from_raw_parts_mut(
                this as *mut u8,
                Self::bytes_len(this),
            ))
        }
    };
}

/// Flat type runtime checking.
pub trait FlatValidate: FlatUnsized {
    /// Check that memory contents of `this` is valid for `Self`.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn validate(this: &Unvalidated<Self>) -> Result<&Self, Error>;
    /// The same as [`Self::validate`] but returns mutable reference.
    fn validate_mut(this: &mut Unvalidated<Self>) -> Result<&mut Self, Error> {
        Self::validate(this)?;
        unsafe { Ok(this.assume_init_mut()) }
    }
}

/// Flat type.
///
/// *If you want to implement this type for your custom type it's recommended to use safe `#[flat]` attribute macro instead.*
///
/// # Safety
///
/// By implementing this trait by yourself you guarantee:
///
/// + `Self` has stable binary representation that will not change in future.
///   (But the representation could be differ across different platforms. If you need stronger guarantees see [`Portable`](`crate::Portable`).)
/// + `Self` don't own any resources outside of it.
/// + `Self` could be trivially copied as bytes. (We cannot require `Self: `[`Copy`] because it `?Sized`.)
/// + All methods of dependent traits have proper implemetation and will not cause an Undefined Behaviour.
pub unsafe trait Flat: FlatBase + FlatUnsized + FlatValidate {}
