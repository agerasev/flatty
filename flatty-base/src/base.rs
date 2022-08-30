use crate::error::{Error, ErrorKind};

/// Basic flat type properties.
pub trait FlatBase {
    /// Align of the type.
    const ALIGN: usize;
    /// Minimal size of of an instance of the type.
    const MIN_SIZE: usize;

    /// Size of an instance of the type.
    fn size(&self) -> usize;
}

/// Check that memory size and alignment are suitable for `Self`.
pub(crate) fn check_size_and_align<T: FlatBase + ?Sized>(mem: &[u8]) -> Result<(), Error> {
    if mem.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize {
                actual: mem.len(),
                required: T::MIN_SIZE,
            },
            position: 0,
        })
    } else if mem.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign { required: T::ALIGN },
            position: 0,
        })
    } else {
        Ok(())
    }
}
