use super::{AsyncReadBuffer, AsyncWriteBuffer, IoBuffer};
use futures::{
    io::{AsyncRead, AsyncWrite},
    ready, Future,
};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

impl<P: AsyncWrite + Unpin> AsyncWriteBuffer for IoBuffer<P> {
    type Error = io::Error;

    fn poll_alloc(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let n = self.buffer.vacant_len();
        if n > 0 {
            self.buffer.advance(n);
        }
        Poll::Ready(Ok(()))
    }

    type WriteAll<'a> = WriteAll<'a, P> where P: 'a;

    fn write_all(&mut self, count: usize) -> Self::WriteAll<'_> {
        WriteAll {
            owner: self,
            pos: 0,
            count,
        }
    }
}

pub struct WriteAll<'a, P: AsyncWrite + Unpin + 'a> {
    owner: &'a mut IoBuffer<P>,
    pos: usize,
    count: usize,
}

impl<'a, P: AsyncWrite + Unpin + 'a> Unpin for WriteAll<'a, P> {}

impl<'a, P: AsyncWrite + Unpin + 'a> Future for WriteAll<'a, P> {
    type Output = io::Result<()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(!self.owner.poisoned);
        while self.pos < self.count {
            let (pos, count) = (self.pos, self.count);
            let (pipe, buffer) = self.owner.split_mut();
            match ready!(Pin::new(pipe).poll_write(cx, &buffer.occupied()[pos..count])) {
                Ok(n) => {
                    if n == 0 {
                        if self.pos != 0 {
                            self.owner.poisoned = true;
                        }
                        return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
                    } else {
                        self.pos += n;
                    }
                }
                Err(e) => {
                    if self.pos != 0 {
                        self.owner.poisoned = true;
                        return Poll::Ready(Err(e));
                    }
                }
            }
        }
        ready!(Pin::new(&mut self.owner.pipe).poll_flush(cx))?;
        self.owner.buffer.clear();
        Poll::Ready(Ok(()))
    }
}

impl<P: AsyncRead + Unpin> AsyncReadBuffer for IoBuffer<P> {
    type Error = io::Error;

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
        let res = ready!(Pin::new(pipe).poll_read(cx, buffer.vacant_mut()));
        if let Ok(n) = res {
            self.buffer.advance(n);
        }
        Poll::Ready(res)
    }

    fn skip(&mut self, count: usize) {
        self.buffer.skip(count);
    }
}
