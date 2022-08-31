use crate::error::{Error, ErrorKind};

/// Basic flat type properties.
pub trait FlatBase {
    /// Align of the type.
    const ALIGN: usize;
    /// Minimal size of an instance of the type.
    const MIN_SIZE: usize;

    /// Size of an instance of the type.
    fn size(&self) -> usize;

    /// Make a pointer to `Self` from bytes without any checks.
    fn ptr_from_bytes(bytes: &[u8]) -> *const Self;

    /// Make a mutable pointer to `Self` from bytes without any checks.
    fn ptr_from_mut_bytes(bytes: &mut [u8]) -> *mut Self;
}

/// Check that memory size and alignment are suitable for `Self`.
pub(crate) fn check_align_and_min_size<T: FlatBase + ?Sized>(mem: &[u8]) -> Result<(), Error> {
    if mem.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })
    } else if mem.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign,
            pos: 0,
        })
    } else {
        Ok(())
    }
}
