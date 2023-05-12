use crate::{
    error::{Error, ErrorKind},
    traits::FlatBase,
};
use core::{ptr, ptr::NonNull};

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

pub unsafe fn slice_ptr_len<T>(slice: *mut [T]) -> usize {
    unsafe { NonNull::new_unchecked(slice) }.len()
}

pub unsafe fn set_slice_ptr_len<T>(bytes: *mut [T], len: usize) -> *mut [T] {
    ptr::slice_from_raw_parts_mut(bytes as *mut T, len)
}

pub unsafe fn offset_slice_ptr_start<T>(slice: *mut [T], count: isize) -> *mut [T] {
    let (ptr, len) = (slice as *mut T, slice_ptr_len(slice));
    ptr::slice_from_raw_parts_mut(ptr.offset(count), (len as isize - count) as usize)
}

#[doc(hidden)]
#[macro_export]
macro_rules! cast_wide_ptr_with_offset {
    ($Type:ty, $ptr:expr, $count:expr $(,)?) => {{
        let wptr = $ptr as *mut [u8];
        let (ptr, len) = (wptr as *mut u8, ::core::ptr::NonNull::new_unchecked(wptr as *mut [u8]).len());
        ::core::ptr::slice_from_raw_parts_mut(ptr.offset($count as isize), len) as *mut $Type
    }};
}
pub use cast_wide_ptr_with_offset;
