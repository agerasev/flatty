//! # Flatty
//!
//! ## TODO
//!
//! + Should we allow [`FlatVec`] items to be non-[`Copy`]?
//! + What if constructed from non-zeroed slice? Should we validate on constructing?
//! + Interpret should return `Result`.

mod prim;
mod util;

pub mod base;
pub mod len;
pub mod sized;
pub mod vec;

//pub mod unsized_enum;

pub use base::Flat;
pub use vec::FlatVec;

//pub use unsized_enum::UnsizedEnum;
