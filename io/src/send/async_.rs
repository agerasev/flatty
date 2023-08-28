use super::{CommonSender, SendError, SendGuard, Sender, UninitSendGuard};
use flatty::{self, prelude::*, utils::alloc::AlignedBytes};
use futures::{
    future::FusedFuture,
    io::AsyncWrite,
    lock::{Mutex, MutexGuard, MutexLockFuture},
    FutureExt,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
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

struct Base<W: AsyncWrite + Unpin> {
    write: Mutex<W>,
    poisoned: AtomicBool,
}

pub struct AsyncSharedSender<M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    base: Arc<Base<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> AsyncSharedSender<M, W> {
    pub fn new(write: W, max_msg_size: usize) -> Self {
        Self {
            base: Arc::new(Base {
                write: Mutex::new(write),
                poisoned: AtomicBool::new(false),
            }),
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc(&mut self) -> UninitSendGuard<'_, M, Self> {
        UninitSendGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> Clone for AsyncSharedSender<M, W> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> CommonSender<M> for AsyncSharedSender<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn poisoned(&self) -> bool {
        self.base.poisoned.load(Ordering::Relaxed)
    }
}

enum SharedSendState<'a, W: AsyncWrite + Unpin> {
    Wait,
    Lock(MutexLockFuture<'a, W>),
    Write(MutexGuard<'a, W>),
}

pub struct SharedSendFuture<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    owner: &'a AsyncSharedSender<M, W>,
    state: Option<SharedSendState<'a, W>>,
    data: &'a [u8],
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Unpin for SharedSendFuture<'a, M, W> {}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Future for SharedSendFuture<'a, M, W> {
    type Output = Result<(), SendError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.take().unwrap();
        let mut poll = true;
        while poll {
            (poll, state) = match state {
                SharedSendState::Wait => (true, SharedSendState::Lock(self.owner.base.write.lock())),
                SharedSendState::Lock(mut lock) => match lock.poll_unpin(cx) {
                    Poll::Pending => (false, SharedSendState::Lock(lock)),
                    Poll::Ready(writer) => {
                        assert!(!self.owner.base.poisoned.load(Ordering::Relaxed));
                        (true, SharedSendState::Write(writer))
                    }
                },
                SharedSendState::Write(mut writer) => match Pin::new(&mut *writer).poll_write(cx, self.data) {
                    Poll::Pending => (false, SharedSendState::Write(writer)),
                    Poll::Ready(Ok(n)) => {
                        if n > 0 {
                            self.data = &self.data[n..];
                            if self.data.is_empty() {
                                return Poll::Ready(Ok(()));
                            } else {
                                (true, SharedSendState::Write(writer))
                            }
                        } else {
                            return Poll::Ready(Err(SendError::Eof));
                        }
                    }
                    Poll::Ready(Err(e)) => {
                        self.owner.base.poisoned.store(true, Ordering::Relaxed);
                        return Poll::Ready(Err(SendError::Io(e)));
                    }
                },
            };
        }
        assert!(self.state.replace(state).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> FusedFuture for SharedSendFuture<'a, M, W> {
    fn is_terminated(&self) -> bool {
        self.state.is_some()
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Send + Unpin> AsyncSender<M> for AsyncSharedSender<M, W> {
    type SendFuture<'a> = SharedSendFuture<'a, M, W> where Self: 'a;
    fn send_buffer(&mut self, count: usize) -> Self::SendFuture<'_> {
        SharedSendFuture {
            owner: self,
            state: Some(SharedSendState::Wait),
            data: &self.buffer[..count],
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
