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
        let mut pos = 0;
        while pos < count {
            match self.pipe.write(&self.buffer.occupied()[pos..count]) {
                Ok(n) => {
                    if n == 0 {
                        if pos != 0 {
                            self.poisoned = true;
                        }
                        return Err(io::ErrorKind::BrokenPipe.into());
                    } else {
                        pos += n;
                    }
                }
                Err(e) => {
                    if pos != 0 {
                        self.poisoned = true;
                        return Err(e);
                    }
                }
            }
        }
        self.buffer.clear();
        Ok(())
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
        let res = self.pipe.read(self.buffer.vacant_mut());
        if let Ok(n) = res {
            self.buffer.advance(n);
        }
        res
    }

    fn skip(&mut self, count: usize) {
        self.buffer.skip(count);
    }
}
