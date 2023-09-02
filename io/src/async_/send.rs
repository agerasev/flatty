use super::{SendError, SendGuard, Sender, UninitSendGuard, WriteBuffer};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use flatty::{self, prelude::*};

pub trait AsyncWriteBuffer: WriteBuffer + Unpin {
    /// Allocate some fixed amount of bytes in the buffer.
    fn poll_alloc(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn alloc(&mut self) -> Alloc<'_, Self> {
        Alloc(self)
    }

    type WriteAllFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;
    /// Send exactly `count` bytes from buffer.
    /// Remaining bytes are discarded.
    fn write_all(&mut self, count: usize) -> Self::WriteAllFuture<'_>;
}

pub struct Alloc<'a, B: AsyncWriteBuffer + ?Sized>(&'a mut B);

impl<'a, B: AsyncWriteBuffer + ?Sized> Future for Alloc<'a, B> {
    type Output = Result<(), B::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_alloc(cx)
    }
}

impl<M: Flat + ?Sized, B: AsyncWriteBuffer> Sender<M, B> {
    pub async fn alloc_(&mut self) -> Result<UninitSendGuard<'_, M, B>, SendError<B::Error>> {
        self.buffer.alloc().await?;
        Ok(UninitSendGuard::new(&mut self.buffer))
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer> SendGuard<'a, M, B> {
    pub fn send_(self) -> B::WriteAllFuture<'a> {
        let size = self.size();
        self.buffer.write_all(size)
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer, const INIT: bool> Unpin for SendGuard<'a, M, B, INIT> {}
