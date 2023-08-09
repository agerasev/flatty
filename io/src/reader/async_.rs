use super::{CommonReader, ReadError, ReadGuard, Reader};
use flatty::Flat;
use futures::io::AsyncRead;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncReader<M: Flat + ?Sized>: CommonReader<M> {
    type ReadFuture<'a>: Future<Output = Result<ReadGuard<'a, M, Self>, ReadError>>
    where
        Self: 'a;

    fn read_message(&mut self) -> Self::ReadFuture<'_>;
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Reader<M, R> {
    fn poll_read_message(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ReadError>> {
        let poll = loop {
            match self.buffer.next_message() {
                Some(result) => break Poll::Ready(result.map(|_| ())),
                None => {
                    if self.buffer.vacant_len() == 0 {
                        assert!(self.buffer.try_extend_vacant());
                    }
                }
            }
            let reader = Pin::new(&mut self.reader);
            match reader.poll_read(cx, self.buffer.vacant_mut()) {
                Poll::Ready(res) => match res {
                    Ok(count) => {
                        if count != 0 {
                            self.buffer.take_vacant(count);
                        } else {
                            break Poll::Ready(Err(ReadError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(ReadError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        poll
    }

    fn take_message(&mut self) -> ReadGuard<'_, M, Self> {
        ReadGuard::new(self)
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> AsyncReader<M> for Reader<M, R> {
    type ReadFuture<'a> = ReadFuture<'a, M, R> where Self: 'a;
    fn read_message(&mut self) -> Self::ReadFuture<'_> {
        ReadFuture { owner: Some(self) }
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for Reader<M, R> {}

pub struct ReadFuture<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    owner: Option<&'a mut Reader<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for ReadFuture<'a, M, R> {}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Future for ReadFuture<'a, M, R> {
    type Output = Result<ReadGuard<'a, M, Reader<M, R>>, ReadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self.owner.take().unwrap();
        match owner.poll_read_message(cx) {
            Poll::Ready(res) => Poll::Ready(res.map(|()| owner.take_message())),
            Poll::Pending => {
                self.owner.replace(owner);
                Poll::Pending
            }
        }
    }
}
