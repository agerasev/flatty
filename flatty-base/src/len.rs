use crate::Flat;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Primitive type that can represent a length of some flat sequence.
///
/// *We don't use [`usize`] instead of all these types here because [`usize`] isn't portable.*
pub trait FlatLen: Flat + Sized + Copy + Add + AddAssign + Sub + SubAssign {
    const MAX_USIZE: usize;
    fn from_usize(n: usize) -> Option<Self>;
    fn into_usize(self) -> usize;
}

impl FlatLen for u8 {
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

impl FlatLen for u16 {
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

impl FlatLen for u32 {
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
