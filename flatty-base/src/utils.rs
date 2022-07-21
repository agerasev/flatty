use core::mem::MaybeUninit;

// TODO: Remove when [`usize::max`] becomes `const`.
pub const fn max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

// TODO: Remove when [`usize::min`] becomes `const`.
pub const fn min(a: usize, b: usize) -> usize {
    if a <= b {
        a
    } else {
        b
    }
}

#[allow(dead_code)]
pub const fn upper_multiple(x: usize, m: usize) -> usize {
    ((x + m - 1) / m) * m
}

#[allow(dead_code)]
pub const fn aligned_add(a: usize, b: usize, m: usize) -> usize {
    upper_multiple(a, m) + b
}

#[allow(dead_code)]
pub const fn aligned_max(a: usize, b: usize, m: usize) -> usize {
    upper_multiple(max(a, b), m)
}

/// Assume that slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const [MaybeUninit<T>] as *const [T])
}

/// Assume that mutable slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

pub enum Never {}
