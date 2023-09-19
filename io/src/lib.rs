#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod common;
pub use common::*;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(all(feature = "blocking", feature = "io"))]
pub use blocking::{IoReceiver, IoSender};
#[cfg(feature = "blocking")]
pub use blocking::{Receiver, Sender};

#[cfg(feature = "async")]
pub mod async_;
#[cfg(all(feature = "async", feature = "io"))]
pub use async_::{IoReceiver as AsyncIoReceiver, IoSender as AsyncIoSender};
#[cfg(feature = "async")]
pub use async_::{Receiver as AsyncReceiver, Sender as AsyncSender};

#[cfg(all(test, feature = "io"))]
mod tests;
