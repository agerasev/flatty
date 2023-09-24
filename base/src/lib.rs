#![no_std]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/// Emplacer-related functionality.
pub mod emplacer;
/// Error type.
pub mod error;
/// Primitive types.
mod primitive;
/// Traits for flat types.
pub mod traits;
/// Utility functions used by macros, so they must be publicly available.
///
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
/// Flat vector itself and its helper types.
pub mod vec;
/// Smart pointer wrapping.
pub mod wrap;
