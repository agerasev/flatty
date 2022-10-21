#[cfg(feature = "std")]
pub mod alloc;
pub mod iter;

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
pub const fn ceil_mul(x: usize, m: usize) -> usize {
    ((x + m - 1) / m) * m
}

/// Biggest number that is both lower or equal to `x` and a multiple of `m`.
pub const fn floor_mul(x: usize, m: usize) -> usize {
    (x / m) * m
}
