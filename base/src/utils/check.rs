use crate::{
    error::{Error, ErrorKind},
    traits::FlatBase,
};

/// Check that memory size and alignment are suitable for `Self`.
pub fn check_align_and_min_size<T: FlatBase + ?Sized>(mem: &[u8]) -> Result<(), Error> {
    if mem.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign,
            pos: 0,
        })
    } else if mem.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })
    } else {
        Ok(())
    }
}
