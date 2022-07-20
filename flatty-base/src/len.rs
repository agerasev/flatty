use crate::prim::FlatPrim;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// # Safety
pub unsafe trait FlatLen: FlatPrim + Add + AddAssign + Sub + SubAssign {
    const MAX_USIZE: usize;
    fn from_usize(n: usize) -> Option<Self>;
    fn into_usize(self) -> usize;
}

unsafe impl FlatLen for u8 {
    const MAX_USIZE: usize = u8::MAX as usize;
    fn from_usize(n: usize) -> Option<Self> {
        if n <= Self::MAX as usize {
            Some(n as Self)
        } else {
            None
        }
    }
    fn into_usize(self) -> usize {
        self as usize
    }
}
unsafe impl FlatLen for u16 {
    const MAX_USIZE: usize = u16::MAX as usize;
    fn from_usize(n: usize) -> Option<Self> {
        if n <= Self::MAX as usize {
            Some(n as Self)
        } else {
            None
        }
    }
    fn into_usize(self) -> usize {
        self as usize
    }
}
unsafe impl FlatLen for u32 {
    const MAX_USIZE: usize = u32::MAX as usize;
    fn from_usize(n: usize) -> Option<Self> {
        if n <= Self::MAX as usize {
            Some(n as Self)
        } else {
            None
        }
    }
    fn into_usize(self) -> usize {
        self as usize
    }
}
