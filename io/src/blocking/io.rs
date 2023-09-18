use super::{ReadBuffer, WriteBuffer};
use crate::IoBuffer;
use std::io::{self, Read, Write};

impl<P: Write> WriteBuffer for IoBuffer<P> {
    type Error = io::Error;

    fn alloc(&mut self) -> Result<(), Self::Error> {
        let n = self.buffer.vacant_len();
        if n > 0 {
            self.buffer.advance(n);
        }
        Ok(())
    }
    fn write_all(&mut self, count: usize) -> Result<(), Self::Error> {
        assert!(!self.poisoned);
        let res = self.pipe.write_all(&self.buffer.occupied()[..count]);
        if res.is_err() {
            self.poisoned = true;
        }
        self.buffer.clear();
        res
    }
}

impl<P: Read> ReadBuffer for IoBuffer<P> {
    type Error = io::Error;

    fn read(&mut self) -> Result<usize, Self::Error> {
        assert!(!self.poisoned);
        if self.buffer.vacant_len() == 0 {
            if self.buffer.preceding_len() > 0 {
                self.buffer.make_contiguous();
            } else {
                return Err(io::ErrorKind::OutOfMemory.into());
            }
        }
        match self.pipe.read(self.buffer.vacant_mut()) {
            Ok(n) => {
                self.buffer.advance(n);
                Ok(n)
            }
            Err(e) => {
                self.poisoned = true;
                Err(e)
            }
        }
    }

    fn skip(&mut self, count: usize) {
        self.buffer.skip(count);
    }
}
