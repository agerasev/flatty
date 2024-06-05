#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
pub mod bytes;
/// Flat string.
pub mod string;
/// Flat vector itself and its helper types.
pub mod vec;
/// Smart pointer wrapping.
pub mod wrap;
