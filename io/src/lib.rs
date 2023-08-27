pub mod reader;
pub mod writer;

#[cfg(feature = "async")]
pub use reader::{AsyncSharedReadGuard, AsyncSharedReader};
pub use reader::{BlockingSharedReadGuard, BlockingSharedReader, ReadError, ReadGuard, Reader};
#[cfg(feature = "async")]
pub use writer::AsyncSharedWriter;
pub use writer::{BlockingSharedWriter, UninitWriteGuard, WriteError, WriteGuard, Writer};

pub mod prelude {
    #[cfg(feature = "async")]
    pub use super::reader::AsyncReader;
    pub use super::reader::BlockingReader;
    #[cfg(feature = "async")]
    pub use super::writer::AsyncWriteGuard;
    #[cfg(feature = "async")]
    pub use super::writer::AsyncWriter;
    pub use super::writer::BlockingWriteGuard;
    pub use super::writer::BlockingWriter;
}

#[cfg(test)]
mod tests;
