use super::{ReadBuffer, Receiver, RecvError, RecvGuard};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use flatty::{error::ErrorKind, Flat};

pub trait AsyncReadBuffer: ReadBuffer + Unpin {
    /// Receive more bytes and put them in the buffer.
    /// Returns the number of received bytes, zero means that channel is closed.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize, Self::Error>>;

    fn read(&mut self) -> Read<'_, Self> {
        Read(self)
    }
}

pub struct Read<'a, B: AsyncReadBuffer + ?Sized>(&'a mut B);

impl<'a, B: AsyncReadBuffer + ?Sized> Future for Read<'a, B> {
    type Output = Result<usize, B::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_read(cx)
    }
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Receiver<M, B> {
    pub async fn recv_(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        while let Err(e) = M::validate(&self.buffer) {
            match e.kind {
                ErrorKind::InsufficientSize => (),
                _ => return Err(RecvError::Parse(e)),
            }
            if self.buffer.read().await.map_err(RecvError::Buffer)? == 0 {
                return Err(RecvError::Closed);
            }
        }
        Ok(RecvGuard::new(&mut self.buffer))
    }
}
