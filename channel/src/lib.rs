mod reader;
mod writer;

pub mod details {
    pub use super::reader::*;
    pub use super::writer::*;
}

#[cfg(feature = "async")]
pub use reader::AsyncReader;
pub use reader::{ReadError, Reader};
#[cfg(feature = "async")]
pub use writer::AsyncWriter;
pub use writer::Writer;

#[cfg(test)]
mod tests;
