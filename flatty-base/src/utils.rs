use core::mem::MaybeUninit;

/// `const` version of [`usize::max`].
pub const fn max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

/// `const` version of [`usize::min`].
pub const fn min(a: usize, b: usize) -> usize {
    if a <= b {
        a
    } else {
        b
    }
}

/// Smallest number that is both greater or equal to `x` and a multiple of `m`.
pub const fn upper_multiple(x: usize, m: usize) -> usize {
    ((x + m - 1) / m) * m
}

/// Biggest number that is both lower or equal to `x` and a multiple of `m`.
pub const fn lower_multiple(x: usize, m: usize) -> usize {
    (x / m) * m
}

/// Assume that slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub(crate) unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const [MaybeUninit<T>] as *const [T])
}

/// Assume that mutable slice of [`MaybeUninit`] is initialized.
///
/// # Safety
///
/// Slice contents must be initialized.
//
// TODO: Remove on `maybe_uninit_slice` stabilization.
pub(crate) unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}
