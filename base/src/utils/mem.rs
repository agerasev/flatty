use crate::{
    error::{Error, ErrorKind},
    traits::FlatBase,
};
use core::slice;

/// Check that memory size and alignment are suitable for `Self`.
pub fn check_align_and_min_size<T: FlatBase + ?Sized>(bytes: &[u8]) -> Result<(), Error> {
    if bytes.as_ptr().align_offset(T::ALIGN) != 0 {
        Err(Error {
            kind: ErrorKind::BadAlign,
            pos: 0,
        })
    } else if bytes.len() < T::MIN_SIZE {
        Err(Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })
    } else {
        Ok(())
    }
}

pub unsafe fn offset_bytes_start(bytes: &[u8], count: isize) -> &[u8] {
    let (ptr, len) = (bytes.as_ptr(), bytes.len());
    slice::from_raw_parts(ptr.offset(count), (len as isize - count) as usize)
}

pub unsafe fn set_bytes_len(bytes: &[u8], len: usize) -> &[u8] {
    slice::from_raw_parts(bytes.as_ptr(), len)
}

#[doc(hidden)]
#[macro_export]
macro_rules! offset_wide_ptr {
    ($Type:ty, $ptr:expr, $count:expr $(,)?) => {{
        let wptr = $ptr as *const [u8];
        let (ptr, len) = (
            wptr as *const u8,
            ::core::ptr::NonNull::new_unchecked(wptr as *mut [u8]).len(),
        );
        ::core::ptr::slice_from_raw_parts(ptr.offset($count as isize), len) as *const $Type
    }};
}
pub use offset_wide_ptr;
