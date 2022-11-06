mod async_;
mod blocking;
mod common;

pub use async_::*;
pub use blocking::*;
pub use common::*;

use std::io;

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Parse(flatty::Error),
    /// Stream has been closed.
    Eof,
}
