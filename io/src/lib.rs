#[cfg(feature = "std")]
extern crate std;

pub mod common;

#[cfg(feature = "async")]
pub mod async_;
#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(all(test, feature = "io"))]
mod tests;
