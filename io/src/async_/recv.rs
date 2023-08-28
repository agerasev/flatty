use super::{CommonReceiver, Receiver, RecvError, RecvGuard};
use flatty::Flat;
use futures::io::AsyncRead;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncReceiver<M: Flat + ?Sized>: CommonReceiver<M> {
    type RecvFuture<'a>: Future<Output = Result<Self::Guard<'a>, RecvError>>
    where
        Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Receiver<M, R> {
    pub(super) fn poll_receive(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), RecvError>> {
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
                            break Poll::Ready(Err(RecvError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(RecvError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        poll
    }

    pub(super) fn take_message(&mut self) -> RecvGuard<'_, M, R> {
        RecvGuard::new(self)
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> AsyncReceiver<M> for Receiver<M, R> {
    type RecvFuture<'a> = RecvFuture<'a, M, R> where Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvFuture { owner: Some(self) }
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for Receiver<M, R> {}

pub struct RecvFuture<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    owner: Option<&'a mut Receiver<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for RecvFuture<'a, M, R> {}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Future for RecvFuture<'a, M, R> {
    type Output = Result<RecvGuard<'a, M, R>, RecvError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self.owner.take().unwrap();
        match owner.poll_receive(cx) {
            Poll::Ready(res) => Poll::Ready(res.map(|()| owner.take_message())),
            Poll::Pending => {
                self.owner.replace(owner);
                Poll::Pending
            }
        }
    }
}
