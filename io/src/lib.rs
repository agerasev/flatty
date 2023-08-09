pub mod reader;
pub mod writer;

//#[cfg(feature = "async")]
//pub use reader::AsyncReader;
pub use reader::{ReadError, ReadGuard, Reader};
#[cfg(feature = "async")]
pub use writer::AsyncSharedWriter;
pub use writer::{BlockingSharedWriter, UninitWriteGuard, WriteGuard, Writer};

pub mod prelude {
    pub use super::reader::{AsyncReader, BlockingReader};
    pub use super::writer::{AsyncWriteGuard, BlockingWriteGuard};
}

#[cfg(test)]
mod tests;
