#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod common;
pub use common::*;

#[cfg(feature = "async")]
mod async_;
#[cfg(feature = "async")]
pub use async_::*;

#[cfg(feature = "blocking")]
mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::*;

#[cfg(all(test, feature = "io"))]
mod tests;
