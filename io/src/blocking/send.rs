use super::{BufSendGuard, BufferSender, SendError, SendGuard, Sender, UninitSendGuard};
use flatty::{self, prelude::*};

pub trait BlockingBufferSender: BufferSender
where
    for<'b> Self::Guard<'b>: BlockingBufSendGuard,
{
    fn alloc(&mut self) -> Result<Self::Guard<'_>, Self::Error>;
}
pub trait BlockingBufSendGuard: BufSendGuard {
    fn send(self, count: usize) -> Result<(), Self::Error>;
}

impl<M: Flat + ?Sized, B: BlockingBufferSender> Sender<M, B>
where
    for<'b> B::Guard<'b>: BlockingBufSendGuard,
{
    pub fn alloc(&mut self) -> Result<UninitSendGuard<'_, M, B>, SendError<B::Error>> {
        Ok(UninitSendGuard::new(self.buf_send.alloc()?))
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingBufferSender> SendGuard<'a, M, B>
where
    for<'b> B::Guard<'b>: BlockingBufSendGuard,
{
    pub fn send(self) -> Result<(), SendError<B::Error>> {
        let size = self.size();
        self.buffer.send(size)
    }
}
