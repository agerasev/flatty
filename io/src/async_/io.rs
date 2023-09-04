use super::{AsyncReadBuffer, AsyncWriteBuffer, IoBuffer};
use futures::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    ready,
};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

pub use futures::io::WriteAll;

impl<P: AsyncWrite + Unpin> AsyncWriteBuffer for IoBuffer<P> {
    fn poll_alloc(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let n = self.buffer.vacant_len();
        if n > 0 {
            self.buffer.advance(n);
        }
        Poll::Ready(Ok(()))
    }

    type WriteAllFuture<'a> = WriteAll<'a, P> where P: 'a;
    fn write_all(&mut self, count: usize) -> Self::WriteAllFuture<'_> {
        let (pipe, buffer) = self.split_mut();
        pipe.write_all(&buffer.occupied()[..count])
    }
}

impl<P: AsyncRead + Unpin> AsyncReadBuffer for IoBuffer<P> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize, io::Error>> {
        assert!(!self.poisoned);
        if self.buffer.vacant_len() == 0 {
            if self.buffer.preceding_len() > 0 {
                self.buffer.make_contiguous();
            } else {
                return Poll::Ready(Err(io::ErrorKind::OutOfMemory.into()));
            }
        }
        let (pipe, buffer) = self.split_mut();
        match ready!(Pin::new(pipe).poll_read(cx, buffer.vacant_mut())) {
            Ok(n) => {
                self.buffer.advance(n);
                Poll::Ready(Ok(n))
            }
            Err(e) => {
                self.poisoned = true;
                Poll::Ready(Err(e))
            }
        }
    }
}
