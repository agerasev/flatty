use super::{ReadBuffer, Receiver, RecvError, RecvGuard};
use flatty::{error::ErrorKind, Flat};

pub trait BlockingReadBuffer: ReadBuffer {
    /// Receive more bytes and put them in the buffer.
    /// Returns the number of received bytes, zero means that channel is closed.
    fn read(&mut self) -> Result<usize, Self::Error>;
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> Receiver<M, B> {
    pub fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        while let Err(e) = M::validate(&self.buffer) {
            match e.kind {
                ErrorKind::InsufficientSize => (),
                _ => return Err(RecvError::Parse(e)),
            }
            if self.buffer.read().map_err(RecvError::Buffer)? == 0 {
                return Err(RecvError::Closed);
            }
        }
        Ok(RecvGuard::new(&mut self.buffer))
    }
}
