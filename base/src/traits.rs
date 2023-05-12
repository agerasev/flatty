use crate::{emplacer::Emplacer, error::Error, utils::mem::check_align_and_min_size};
use core::{
    mem::{align_of, size_of},
    slice,
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
/// *For now has to be implemented for all [`Flat`] types because there is no mutually exclusive traits in Rust yet.*
pub unsafe trait FlatUnsized: FlatBase {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;

    fn ptr_from_bytes(bytes: &[u8]) -> *const Self;
    unsafe fn ptr_to_bytes<'a>(this: *const Self) -> &'a [u8];

    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*Self::ptr_from_bytes(bytes)
    }
    unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut Self {
        &mut *(Self::ptr_from_bytes(bytes) as *mut _)
    }
    fn as_bytes(&self) -> &[u8] {
        unsafe { Self::ptr_to_bytes(self as *const _) }
    }
    /// # Safety
    ///
    /// Modification of returned bytes must not make `self` invalid.
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        #[allow(clippy::cast_ref_to_mut)]
        unsafe {
            &mut *(Self::ptr_to_bytes(self as *const _) as *const _ as *mut _)
        }
    }

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn new_in_place<I: Emplacer<Self>>(bytes: &mut [u8], emplacer: I) -> Result<&mut Self, Error> {
        emplacer.emplace(bytes)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(bytes) })
    }
    fn assign_in_place<I: Emplacer<Self>>(&mut self, emplacer: I) -> Result<&mut Self, Error> {
        unsafe {
            let bytes = self.as_mut_bytes();
            emplacer.emplace_unchecked(bytes)?;
            Ok(Self::from_mut_bytes_unchecked(bytes))
        }
    }
}

/// Flat type runtime checking.
pub unsafe trait FlatValidate: FlatUnsized {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error>;

    unsafe fn validate_ptr(this: *const Self) -> Result<(), Error> {
        unsafe { Self::validate_unchecked(Self::ptr_to_bytes(this)) }
    }

    /// Check that memory contents of `this` is valid for `Self`.
    fn validate(bytes: &[u8]) -> Result<(), Error> {
        check_align_and_min_size::<Self>(bytes)?;
        unsafe { Self::validate_unchecked(bytes) }
    }

    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        Self::validate(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }
    fn from_mut_bytes(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Self::validate(bytes)?;
        Ok(unsafe { Self::from_mut_bytes_unchecked(bytes) })
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
///   (But the representation could be differ across different platforms. If you need stronger guarantees consider using `Portable` types.)
/// + `Self` don't own any resources outside of it.
/// + `Self` could be trivially copied as bytes. (We cannot require `Self: `[`Copy`] because it `?Sized`.)
/// + All methods of dependent traits have proper implemetation and will not cause an Undefined Behaviour.
pub unsafe trait Flat: FlatBase + FlatUnsized + FlatValidate {}

/// Statically-sized flat type.
///
/// # Safety
///
/// `SIZE` must match `Self` size.
pub unsafe trait FlatSized: FlatUnsized + Sized {
    /// Static size of the type.
    const SIZE: usize = size_of::<Self>();
}

unsafe impl<T: Flat> FlatSized for T {}

unsafe impl<T: FlatSized> FlatBase for T {
    const ALIGN: usize = align_of::<Self>();

    const MIN_SIZE: usize = Self::SIZE;

    fn size(&self) -> usize {
        Self::SIZE
    }
}

unsafe impl<T: FlatSized> FlatUnsized for T {
    type AlignAs = T;

    fn ptr_from_bytes(bytes: &[u8]) -> *const Self {
        bytes.as_ptr() as *const Self
    }
    unsafe fn ptr_to_bytes<'a>(this: *const Self) -> &'a [u8] {
        slice::from_raw_parts(this as *const u8, Self::SIZE)
    }
}

/// Flat types that can be initialized to default state.
///
/// # Safety
///
/// Methods must properly initialize memory.
pub trait FlatDefault: Flat {
    type DefaultEmplacer: Emplacer<Self>;

    /// Initialize uninitialized memory into valid default state.
    ///
    /// This method returned `Ok` must guaratee that `this` could be safely transmuted to `Self`.
    fn default_emplacer() -> Self::DefaultEmplacer;

    /// Create a new instance of `Self` initializing raw memory into default state of `Self`.
    fn default_in_place(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Self::new_in_place(bytes, Self::default_emplacer())
    }
}

impl<T: Flat + Default> FlatDefault for T {
    type DefaultEmplacer = Self;

    fn default_emplacer() -> Self::DefaultEmplacer {
        Self::default()
    }
}
