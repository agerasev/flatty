use std::mem::align_of;

#[derive(Clone, PartialEq, Debug)]
pub enum InterpretError {
    InsufficientSize,
    BadAlign,
    InvalidState,
}

pub trait FlatBase {
    /// Align of the type.
    const ALIGN: usize = align_of::<Self::AlignAs>();
    /// Sized type that has the same alignment as [`Self`].
    type AlignAs: Sized;

    const MIN_SIZE: usize;
    /// Dynamic size of the type.
    fn size(&self) -> usize;

    fn check_size_and_align(mem: &[u8]) -> Result<(), InterpretError> {
        if mem.len() < Self::MIN_SIZE {
            Err(InterpretError::InsufficientSize)
        } else if mem.as_ptr().align_offset(Self::ALIGN) != 0 {
            Err(InterpretError::BadAlign)
        } else {
            Ok(())
        }
    }

    /// Metadata to store in wide pointer. Used only for unsized types.
    fn _ptr_metadata(mem: &[u8]) -> usize;
}

pub trait FlatInit: FlatBase {
    /// Initializer of the `Self` instance.
    type Init: Sized;
    /// Create a new instance of `Self` onto raw memory.
    fn init(mem: &mut [u8], init: Self::Init) -> Result<&mut Self, InterpretError> {
        Self::check_size_and_align(mem)?;
        Ok(unsafe { Self::init_unchecked(mem, init) })
    }
    /// Create a new default instance of `Self` onto raw memory.
    fn init_default(mem: &mut [u8]) -> Result<&mut Self, InterpretError>
    where
        Self::Init: Default,
    {
        Self::init(mem, Self::Init::default())
    }
    /// Initialize without checks.
    ///
    /// # Safety
    unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self;

    /// Validate memory before interpretation.
    fn pre_validate(mem: &[u8]) -> Result<(), InterpretError>;
    /// Validate memory after interpretation.
    fn post_validate(&self) -> Result<(), InterpretError>;

    /// Interpret a previously iniailized memory as an instance of `Self`.
    fn interpret(mem: &[u8]) -> Result<&Self, InterpretError> {
        Self::check_size_and_align(mem)?;
        Self::pre_validate(mem)?;
        let self_ = unsafe { Self::interpret_unchecked(mem) };
        self_.post_validate()?;
        Ok(self_)
    }
    /// The same as [`interpret`](`Self::interpret`) but provides a mutable reference.
    fn interpret_mut(mem: &mut [u8]) -> Result<&mut Self, InterpretError> {
        Self::check_size_and_align(mem)?;
        Self::pre_validate(mem)?;
        let self_ = unsafe { Self::interpret_mut_unchecked(mem) };
        self_.post_validate()?;
        Ok(self_)
    }

    /// Interpret without checks.
    ///
    /// # Safety
    unsafe fn interpret_unchecked(mem: &[u8]) -> &Self;
    /// Interpret without checks providing mutable reference.
    ///
    ///  # Safety
    unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self;
}

/// Flat type.
///
/// # Safety
///
/// The type must have stable binary representation.
pub unsafe trait Flat: FlatBase + FlatInit {}
