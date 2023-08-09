use super::{CommonReader, ReadError, ReadGuard, Reader};
use flatty::Flat;
use std::io::Read;

pub trait BlockingReader<M: Flat + ?Sized>: CommonReader<M> {
    fn read_message(&mut self) -> Result<ReadGuard<'_, M, Self>, ReadError>;
}

impl<M: Flat + ?Sized, R: Read> BlockingReader<M> for Reader<M, R> {
    fn read_message(&mut self) -> Result<ReadGuard<'_, M, Self>, ReadError> {
        loop {
            match self.buffer.next_message() {
                Some(result) => break result.map(|_| ()),
                None => {
                    if self.buffer.vacant_len() == 0 {
                        assert!(self.buffer.try_extend_vacant());
                    }
                }
            }
            match self.reader.read(self.buffer.vacant_mut()) {
                Ok(count) => {
                    if count != 0 {
                        self.buffer.take_vacant(count);
                    } else {
                        break Err(ReadError::Eof);
                    }
                }
                Err(err) => break Err(ReadError::Io(err)),
            }
        }
        .map(|()| ReadGuard::new(self))
    }
}
