//! # Flatty
//!
//! ## TODO
//!
//! + Should we allow [`FlatVec`] items to be non-[`Copy`]?
//! + What if constructed from non-zeroed slice? Should we validate on constructing?
//! + Interpret should return `Result`.

mod prim;

pub mod base;
pub mod error;
pub mod len;
pub mod sized;
pub mod utils;
pub mod vec;

pub use base::{Flat, FlatBase, FlatInit};
pub use error::InterpretError;
pub use sized::FlatSized;
pub use vec::FlatVec;

#[cfg(test)]
mod tests;
