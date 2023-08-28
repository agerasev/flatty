mod buffer;
mod recv;
mod send;

pub mod shared;

pub use recv::*;
pub use send::*;

pub mod prelude {
    pub use super::{CommonReceiver, CommonSender};
}
