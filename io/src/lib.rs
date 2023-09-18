#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod common;
pub use common::*;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::{IoReceiver, IoSender, Receiver, Sender};

#[cfg(feature = "async")]
pub mod async_;
#[cfg(feature = "async")]
pub use async_::{IoReceiver as AsyncIoReceiver, IoSender as AsyncIoSender, Receiver as AsyncReceiver, Sender as AsyncSender};

#[cfg(all(test, feature = "io"))]
mod tests;
