#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod array;
mod base;
mod cast;
mod default;
mod marker;
mod maybe_unsized;
mod prim;
mod sized;

pub mod error;
pub mod iter;
//pub mod iter;
pub mod mem;
/// Utuility functions used by macros, so they must be publicly available.
///
/// *Please, don't use them by yourself because they aren't stable.*
pub mod utils;
/// Flat vector itself and its helper types.
pub mod vec;

/// Flat type.
///
/// *If you want to implement this type for your custom type it's recommended to use safe `make_flat` macro instead.*
///
/// # Safety
///
/// By implementing this trait by yourself you guarantee:
///
/// + `Self` has stable binary representation that will not change in future.
///   (But the representation could be differ across different platforms. If you need such a guarantee see [`Portable`].)
/// + `Self` don't own any resources outside of it.
/// + `Self` could be trivially copied as bytes. (We cannot require `Self: `[`Copy`] because it `?Sized`.)
/// + All `Flat*` traits implemetation for `Self` will not cause an Undefined Behaviour.
pub unsafe trait Flat: FlatBase + FlatMaybeUnsized + FlatCast {}

pub use base::FlatBase;
pub use cast::FlatCast;
pub use default::FlatDefault;
pub use error::{Error, ErrorKind};
pub use maybe_unsized::FlatMaybeUnsized;
pub use sized::FlatSized;
pub use vec::FlatVec;

pub mod prelude {
    pub use super::{Flat, FlatBase, FlatCast, FlatDefault, FlatMaybeUnsized, FlatSized};
}
