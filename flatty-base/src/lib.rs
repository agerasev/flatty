//! # Flatty
//!
//! ## TODO
//!
//! + Should we allow [`FlatVec`] items to be non-[`Copy`]?
//! + What if constructed from non-zeroed slice? Should we validate on constructing?
//! + Interpret should return `Result`.

mod error;
mod prim;
mod util;

pub mod len;
pub mod sized;
pub mod traits;
pub mod vec;

pub use error::InterpretError;
pub use sized::FlatSized;
pub use traits::{Flat, FlatBase, FlatInit};
pub use vec::FlatVec;

#[cfg(test)]
mod tests;
