#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod base;
mod cast;
mod error;
mod sized;
/*
mod marker;
mod prim;

/// Flat collections.
pub mod collections;
/// Primitive portable types.
pub mod portable;
/// Utuility functions used by macros, so they must be publicly available.
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
*/

/// Flat type.
///
/// *If you want to implement this type for your custom type it's recommended to use safe `make_flat` macro instead.*
///
/// # Safety
///
/// By implementing this trait you guarantee that the type:
///
/// + Have stable binary representation that will not change in future.
/// + Don't own any resources outside of itself.
/// + Could be trivially copied as bytes. (We cannot require [`Copy`] because the type is [`?Sized`].)
pub unsafe trait Flat: FlatBase + FlatCast {}

pub use base::FlatBase;
pub use cast::FlatCast;
/*
pub use collections::vec::FlatVec;
pub use error::Error;
pub use portable::Portable;
pub use sized::FlatSized;

pub mod prelude {
    pub use super::{Flat, FlatBase, FlatInit, FlatSized, FlatUnsized, Portable};
}
*/
