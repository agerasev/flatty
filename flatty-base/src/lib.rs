//! # Flatty
//!
//! ## TODO
//!
//! + Should we allow [`FlatVec`] items to be non-[`Copy`]?
//! + What if constructed from non-zeroed slice? Should we validate on constructing?
//! + Interpret should return `Result`.

mod array;
mod base;
mod error;
mod len;
mod marker;
mod prim;
mod sized;

/// Utuility functions used by macros, so they must be publicly available.
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
/// Flat vector itself and its helper types.
pub mod vec;

pub use base::{Flat, FlatBase, FlatInit, FlatUnsized};
pub use error::Error;
pub use len::FlatLen;
pub use sized::FlatSized;
pub use vec::FlatVec;
