use crate::error::Error;

/// Basic functionality for flat types.
pub trait FlatBase {
    /// Align of the type.
    const ALIGN: usize;

    /// Minimal size of of an instance of the type.
    const MIN_SIZE: usize;
    /// Size of an instance of the type.
    fn size(&self) -> usize;

    /// Check that memory size and alignment are suitable for `Self`.
    fn check_size_and_align(mem: &[u8]) -> Result<(), Error> {
        if mem.len() < Self::MIN_SIZE {
            Err(Error::InsufficientSize)
        } else if mem.as_ptr().align_offset(Self::ALIGN) != 0 {
            Err(Error::BadAlign)
        } else {
            Ok(())
        }
    }
}

/// Construction of flat types.
pub trait FlatInit: FlatBase {
    /// Initializer of the `Self` instance.
    type Init: Sized;
    /// Create a new instance of `Self` onto raw memory.
    fn placement_new(mem: &mut [u8], init: Self::Init) -> Result<&mut Self, Error> {
        Self::check_size_and_align(mem)?;
        Ok(unsafe { Self::placement_new_unchecked(mem, init) })
    }
    /// Create a new default instance of `Self` onto raw memory.
    fn placement_default(mem: &mut [u8]) -> Result<&mut Self, Error>
    where
        Self::Init: Default,
    {
        Self::placement_new(mem, Self::Init::default())
    }
    /// Initialize without checks.
    ///
    /// # Safety
    ///
    /// Size, alignment and specific type requirements must be ok.
    unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self;

    /// Validate memory before interpretation.
    fn pre_validate(mem: &[u8]) -> Result<(), Error>;
    /// Validate memory after interpretation.
    fn post_validate(&self) -> Result<(), Error>;

    /// Interpret a previously iniailized memory as an instance of `Self`.
    fn reinterpret(mem: &[u8]) -> Result<&Self, Error> {
        Self::check_size_and_align(mem)?;
        Self::pre_validate(mem)?;
        let self_ = unsafe { Self::reinterpret_unchecked(mem) };
        self_.post_validate()?;
        Ok(self_)
    }
    /// The same as [`reinterpret`](`Self::reinterpret`) but provides a mutable reference.
    fn reinterpret_mut(mem: &mut [u8]) -> Result<&mut Self, Error> {
        Self::check_size_and_align(mem)?;
        Self::pre_validate(mem)?;
        let self_ = unsafe { Self::reinterpret_mut_unchecked(mem) };
        self_.post_validate()?;
        Ok(self_)
    }

    /// Interpret without checks.
    ///
    /// # Safety
    ///
    /// Memory must have suitable size and align for `Self` and its contents must be valid.  
    unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self;
    /// Interpret without checks providing mutable reference.
    ///
    /// # Safety
    ///
    /// Memory must have suitable size and align for `Self` and its contents must be valid.  
    unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self;
}

/// Dynamically-sized flat type.
///
/// *For now it must be implemented for all flat types until negative trait bounds supported.*
pub trait FlatUnsized: FlatBase {
    /// Sized type that has the same alignment as [`Self`].
    type AlignAs: Sized;

    /// Metadata to store in wide pointer.
    fn ptr_metadata(mem: &[u8]) -> usize;
}

/// Marker trait for flat types.
///
/// *If you want to implement this type for your custom type it's recommended to use safe `make_flat` macro instead.*
///
/// # Safety
///
/// The type must have portable binary representation.
pub unsafe trait Flat: FlatBase + FlatInit + FlatUnsized {}
