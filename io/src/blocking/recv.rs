use super::{BufRecvGuard, BufferReceiver, Receiver, RecvError, RecvGuard};
use flatty::{error::ErrorKind, Flat};

pub trait BlockingBufferReceiver: BufferReceiver {
    fn recv(&mut self) -> Result<Self::Guard<'_>, Self::Error>;
}

impl<M: Flat + ?Sized, B: BlockingBufferReceiver> Receiver<M, B> {
    pub fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        let mut buffer = self.buf_recv.recv().map_err(RecvError::Buffer)?;
        Ok(RecvGuard::new(loop {
            match M::validate(&buffer) {
                Err(e) => match e.kind {
                    ErrorKind::InsufficientSize => buffer.extend().map_err(RecvError::Buffer)?,
                    _ => return Err(RecvError::Parse(e)),
                },
                Ok(()) => break buffer,
            }
        }))
    }
}
