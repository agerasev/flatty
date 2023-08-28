pub mod recv;
pub mod send;

#[cfg(feature = "async")]
pub use recv::AsyncSharedReceiver;
pub use recv::{BlockingSharedReceiver, Receiver, RecvError};
#[cfg(feature = "async")]
pub use send::AsyncSharedSender;
pub use send::{BlockingSharedSender, SendError, Sender};

pub mod prelude {
    #[cfg(feature = "async")]
    pub use super::recv::AsyncReceiver;
    pub use super::recv::BlockingReceiver;
    #[cfg(feature = "async")]
    pub use super::send::AsyncSendGuard;
    pub use super::send::BlockingSendGuard;
}

#[cfg(test)]
mod tests;
