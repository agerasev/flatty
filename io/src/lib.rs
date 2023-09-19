#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod common;
#[doc(inline)]
pub use common::*;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(all(feature = "blocking", feature = "io"))]
#[doc(inline)]
pub use blocking::{IoReceiver, IoSender};
#[cfg(feature = "blocking")]
#[doc(inline)]
pub use blocking::{ReadBuffer, Receiver, Sender, WriteBuffer};

#[cfg(feature = "async")]
pub mod async_;
#[cfg(feature = "async")]
#[doc(inline)]
pub use async_::{AsyncReadBuffer, AsyncWriteBuffer, Receiver as AsyncReceiver, Sender as AsyncSender};
#[cfg(all(feature = "async", feature = "io"))]
#[doc(inline)]
pub use async_::{IoReceiver as AsyncIoReceiver, IoSender as AsyncIoSender};

#[cfg(all(test, feature = "io"))]
mod tests;
