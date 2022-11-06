use super::{AbstractReader, ReadBuffer, ReadError, ReadGuard};
use flatty::Portable;
use std::io::Read;

pub struct Reader<M: Portable + ?Sized, R: Read> {
    reader: R,
    buffer: ReadBuffer<M>,
}

impl<M: Portable + ?Sized, R: Read> Reader<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: ReadBuffer::new(max_msg_size),
        }
    }

    pub fn read_message(&mut self) -> Result<ReadGuard<'_, M, Self>, ReadError> {
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

impl<M: Portable + ?Sized, R: Read> AbstractReader<M> for Reader<M, R> {
    fn buffer(&self) -> &ReadBuffer<M> {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut ReadBuffer<M> {
        &mut self.buffer
    }
}
