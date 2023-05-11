#![no_std]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "std")]
extern crate std;

pub mod emplacer;
pub mod error;
mod primitive;
pub mod traits;
/// Utility functions used by macros, so they must be publicly available.
///
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
///// Flat vector itself and its helper types.
pub mod vec;
