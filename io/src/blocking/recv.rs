use super::{ReadBuffer, Receiver, RecvError, RecvGuard};
use flatty::{error::ErrorKind, Flat};

pub trait BlockingReadBuffer: ReadBuffer {
    /// Receive at least `extra` more bytes and put them in the buffer.
    ///
    /// Returns the number of received bytes.
    /// If `extra` is non-zero then returned zero means that buffer is closed.
    fn read(&mut self, extra: usize) -> Result<usize, Self::Error>;
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> Receiver<M, B> {
    pub fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        while let Err(e) = M::validate(&self.buffer) {
            match e.kind {
                ErrorKind::InsufficientSize => (),
                _ => return Err(RecvError::Parse(e)),
            }
            let extra = M::MIN_SIZE.saturating_sub(self.buffer.len()).max(1);
            if self.buffer.read(extra).map_err(RecvError::Buffer)? == 0 {
                return Err(RecvError::Closed);
            }
        }
        Ok(RecvGuard::new(&mut self.buffer))
    }
}
