use core::mem::MaybeUninit;

// TODO: Remove when [`usize::max`] becomes `const`.
pub const fn usize_max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

// TODO: Remove when [`usize::min`] becomes `const`.
#[allow(dead_code)]
pub const fn usize_min(a: usize, b: usize) -> usize {
    if a <= b {
        a
    } else {
        b
    }
}

// TODO: Remove on `maybe_uninit_slice` stabilization.
pub unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const [MaybeUninit<T>] as *const [T])
}

// TODO: Remove on `maybe_uninit_slice` stabilization.
pub unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

pub enum Never {}
