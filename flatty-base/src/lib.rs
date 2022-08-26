#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod base;
mod error;
mod marker;
mod prim;
mod sized;

/// Flat collections.
pub mod collections;
/// Primitive portable types.
pub mod portable;
/// Utuility functions used by macros, so they must be publicly available.
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;

pub use base::{Flat, FlatBase, FlatInit, FlatUnsized};
pub use collections::vec::FlatVec;
pub use error::Error;
pub use portable::Portable;
pub use sized::FlatSized;

pub mod prelude {
    pub use super::{Flat, FlatBase, FlatInit, FlatSized, FlatUnsized, Portable};
}
