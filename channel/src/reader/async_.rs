use super::{AbstractReader, ReadBuffer, ReadError, ReadGuard};
use flatty::Portable;
use futures::io::AsyncRead;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct AsyncReader<M: Portable + ?Sized, R: AsyncRead + Unpin> {
    reader: R,
    buffer: Option<ReadBuffer<M>>,
}

impl<M: Portable + ?Sized, R: AsyncRead + Unpin> AsyncReader<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: Some(ReadBuffer::new(max_msg_size)),
        }
    }

    fn poll_read_message(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ReadError>> {
        let mut buffer = self.buffer.take().unwrap();
        let poll = loop {
            match buffer.next_message() {
                Some(result) => break Poll::Ready(result.map(|_| ())),
                None => {
                    if buffer.vacant_len() == 0 {
                        assert!(buffer.try_extend_vacant());
                    }
                }
            }
            let reader = Pin::new(&mut self.reader);
            match reader.poll_read(cx, buffer.vacant_mut()) {
                Poll::Ready(res) => match res {
                    Ok(count) => {
                        if count != 0 {
                            buffer.take_vacant(count);
                        } else {
                            break Poll::Ready(Err(ReadError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(ReadError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        assert!(self.buffer.replace(buffer).is_none());
        poll
    }

    fn take_message(&mut self) -> ReadGuard<'_, M, Self> {
        ReadGuard::new(self)
    }

    pub fn read_message(&mut self) -> ReadFuture<'_, M, R> {
        ReadFuture { owner: Some(self) }
    }
}

impl<M: Portable + ?Sized, R: AsyncRead + Unpin> AbstractReader<M> for AsyncReader<M, R> {
    fn buffer(&self) -> &ReadBuffer<M> {
        self.buffer.as_ref().unwrap()
    }
    fn buffer_mut(&mut self) -> &mut ReadBuffer<M> {
        self.buffer.as_mut().unwrap()
    }
}

impl<M: Portable + ?Sized, R: AsyncRead + Unpin> Unpin for AsyncReader<M, R> {}

pub struct ReadFuture<'a, M: Portable + ?Sized, R: AsyncRead + Unpin> {
    owner: Option<&'a mut AsyncReader<M, R>>,
}

impl<'a, M: Portable + ?Sized, R: AsyncRead + Unpin> Unpin for ReadFuture<'a, M, R> {}

impl<'a, M: Portable + ?Sized, R: AsyncRead + Unpin> Future for ReadFuture<'a, M, R> {
    type Output = Result<ReadGuard<'a, M, AsyncReader<M, R>>, ReadError>;

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
