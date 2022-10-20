#![no_std]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "std")]
extern crate std;

mod array;
mod default;
mod emplacer;
mod error;
mod flat;
mod marker;
mod prim;
mod sized;

pub mod mem;
/// Utuility functions used by macros, so they must be publicly available.
///
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
/// Flat vector itself and its helper types.
pub mod vec;

pub use default::FlatDefault;
pub use emplacer::{Emplacer, NeverEmplacer};
pub use error::{Error, ErrorKind};
pub use flat::{Flat, FlatBase, FlatCheck, FlatUnsized};
pub use sized::FlatSized;
pub use vec::FlatVec;

pub mod prelude {
    pub use super::{Flat, FlatBase, FlatCheck, FlatDefault, FlatSized, FlatUnsized};
}
