use super::{SendError, SendGuard, Sender, UninitSendGuard, WriteBuffer};
use flatty::{self, prelude::*};

pub trait BlockingWriteBuffer: WriteBuffer {
    /// Allocate some fixed amount of bytes in the buffer.
    fn alloc(&mut self) -> Result<(), Self::Error>;
    /// Send exactly `count` bytes from buffer.
    /// Remaining bytes are discarded.
    fn write(&mut self, count: usize) -> Result<(), Self::Error>;
}

impl<M: Flat + ?Sized, B: BlockingWriteBuffer> Sender<M, B> {
    pub fn alloc(&mut self) -> Result<UninitSendGuard<'_, M, B>, SendError<B::Error>> {
        self.buffer.alloc()?;
        Ok(UninitSendGuard::new(&mut self.buffer))
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer> SendGuard<'a, M, B> {
    pub fn send(self) -> Result<(), SendError<B::Error>> {
        let size = self.size();
        self.buffer.write(size)
    }
}
