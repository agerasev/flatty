use std::mem::align_of;

pub trait FlatBase {
    /// Align of the type.
    const ALIGN: usize = align_of::<Self::AlignAs>();
    /// Sized type that has the same alignment as [`Self`].
    type AlignAs: Sized;

    const MIN_SIZE: usize;
    /// Dynamic size of the type.
    fn size(&self) -> usize;

    fn check_size_and_align(mem: &[u8]) -> bool {
        mem.len() >= Self::MIN_SIZE && mem.as_ptr().align_offset(Self::ALIGN) == 0
    }
}

pub trait FlatInit: FlatBase {
    /// Initializer of the `Self` instance.
    type Init: Sized;
    /// Create a new instance of `Self` onto raw memory.
    fn init(mem: &mut [u8], init: Self::Init) -> &mut Self;
    /// Create a new default instance of `Self` onto raw memory.
    fn init_default(mem: &mut [u8]) -> &mut Self
    where
        Self::Init: Default,
    {
        Self::init(mem, Self::Init::default())
    }

    fn validate(&self) -> bool;
    /// Interpret a previously iniailized memory as an instance of `Self`.
    ///
    /// Panics if the memory state is invalid.
    fn interpret(mem: &[u8]) -> &Self;
    /// The same as [`interpret`](`Self::interpret`) but provides a mutable reference.
    fn interpret_mut(mem: &mut [u8]) -> &mut Self;

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
