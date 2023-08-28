use super::{CommonReceiver, Receiver, RecvError, RecvGuard};
use flatty::Flat;
use std::io::Read;

pub trait BlockingReceiver<M: Flat + ?Sized>: CommonReceiver<M> {
    fn recv(&mut self) -> Result<Self::RecvGuard<'_>, RecvError>;
}

impl<M: Flat + ?Sized, R: Read> BlockingReceiver<M> for Receiver<M, R> {
    fn recv(&mut self) -> Result<Self::RecvGuard<'_>, RecvError> {
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
                        break Err(RecvError::Eof);
                    }
                }
                Err(err) => break Err(RecvError::Io(err)),
            }
        }
        .map(|()| RecvGuard::new(self))
    }
}
