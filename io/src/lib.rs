pub mod reader;
pub mod writer;

#[cfg(feature = "async")]
pub use reader::AsyncSharedReader;
pub use reader::{BlockingSharedReader, ReadError, Reader};
#[cfg(feature = "async")]
pub use writer::AsyncSharedWriter;
pub use writer::{BlockingSharedWriter, WriteError, Writer};

pub mod prelude {
    #[cfg(feature = "async")]
    pub use super::reader::AsyncReader;
    pub use super::reader::BlockingReader;
    #[cfg(feature = "async")]
    pub use super::writer::AsyncWriteGuard;
    pub use super::writer::BlockingWriteGuard;
}

#[cfg(test)]
mod tests;
