use crate::Flat;

/// # Safety
pub unsafe trait FlatSize: Flat + Sized + Copy {
    const MAX_USIZE: usize;
    fn from_usize(n: usize) -> Option<Self>;
    fn into_usize(self) -> usize;
}

unsafe impl FlatSize for u8 {
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
unsafe impl FlatSize for u16 {
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
unsafe impl FlatSize for u32 {
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
