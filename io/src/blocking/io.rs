use super::{BlockingReadBuffer, BlockingWriteBuffer, IoBuffer};
use std::io::{self, Read, Write};

impl<P: Write> BlockingWriteBuffer for IoBuffer<P> {
    fn alloc(&mut self) -> Result<(), Self::Error> {
        let n = self.buffer.vacant_len();
        if n > 0 {
            self.buffer.advance(n);
            Ok(())
        } else {
            Err(io::ErrorKind::OutOfMemory.into())
        }
    }
    fn write(&mut self, count: usize) -> Result<(), Self::Error> {
        assert!(!self.poisoned);
        let mut data = &self.buffer.occupied()[..count];
        let res = loop {
            match self.pipe.write(data) {
                Ok(n) => {
                    if n > 0 {
                        data = &data[n..];
                        if data.is_empty() {
                            break Ok(());
                        }
                    } else {
                        break Err(io::ErrorKind::UnexpectedEof.into());
                    }
                }
                Err(e) => {
                    self.poisoned = true;
                    break Err(e);
                }
            }
        };
        self.buffer.clear();
        res
    }
}

impl<P: Read> BlockingReadBuffer for IoBuffer<P> {
    fn read(&mut self, extra: usize) -> Result<usize, Self::Error> {
        assert!(!self.poisoned);
        if self.buffer.vacant_len() < extra {
            if self.buffer.free_len() >= extra {
                self.buffer.make_contiguous();
            } else {
                return Err(io::ErrorKind::OutOfMemory.into());
            }
        }
        let mut count = 0;
        while count < extra {
            match self.pipe.read(self.buffer.vacant_mut()) {
                Ok(n) => {
                    if n != 0 {
                        self.buffer.advance(n);
                        count += n;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    self.poisoned = true;
                    return Err(e);
                }
            }
        }
        Ok(count)
    }
}
