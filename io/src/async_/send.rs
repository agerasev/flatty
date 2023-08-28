use crate::common::{CommonSender, SendError, SendGuard, Sender, UninitSendGuard};
use flatty::{self, prelude::*};
use futures::{future::FusedFuture, io::AsyncWrite};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncSender<M: Flat + ?Sized>: CommonSender<M> {
    type SendFuture<'a>: Future<Output = Result<(), SendError>> + Send + 'a
    where
        Self: 'a;
    fn send_buffer(&mut self, count: usize) -> Self::SendFuture<'_>;
}

pub trait AsyncSendGuard<'a> {
    type SendFuture: Future<Output = Result<(), SendError>>;
    fn send(self) -> Self::SendFuture;
}

pub struct SendFuture<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    write: Option<&'a mut W>,
    data: &'a [u8],
    poisoned: &'a mut bool,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Unpin for SendFuture<'a, M, W> {}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Future for SendFuture<'a, M, W> {
    type Output = Result<(), SendError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut write = self.write.take().unwrap();
        loop {
            match Pin::new(&mut write).poll_write(cx, self.data) {
                Poll::Pending => break,
                Poll::Ready(Ok(n)) => {
                    if n > 0 {
                        self.data = &self.data[n..];
                        if self.data.is_empty() {
                            return Poll::Ready(Ok(()));
                        }
                    } else {
                        return Poll::Ready(Err(SendError::Eof));
                    }
                }
                Poll::Ready(Err(e)) => {
                    *(self.poisoned) = true;
                    return Poll::Ready(Err(SendError::Io(e)));
                }
            }
        }
        assert!(self.write.replace(write).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> FusedFuture for SendFuture<'a, M, W> {
    fn is_terminated(&self) -> bool {
        self.write.is_none()
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Send + Unpin> AsyncSender<M> for Sender<M, W> {
    type SendFuture<'a> = SendFuture<'a, M, W> where Self: 'a;

    fn send_buffer(&mut self, count: usize) -> Self::SendFuture<'_> {
        assert!(!self.poisoned);
        SendFuture {
            write: Some(&mut self.write),
            data: &self.buffer[..count],
            poisoned: &mut self.poisoned,
            _ghost: PhantomData,
        }
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncSender<M>> AsyncSendGuard<'a> for SendGuard<'a, M, O> {
    type SendFuture = <O as AsyncSender<M>>::SendFuture<'a>;
    fn send(self) -> Self::SendFuture {
        self.owner.send_buffer(self.size())
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncSender<M>> Unpin for UninitSendGuard<'a, M, O> {}
impl<'a, M: Flat + ?Sized, O: AsyncSender<M>> Unpin for SendGuard<'a, M, O> {}
