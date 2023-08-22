use super::{CommonWriter, UninitWriteGuard, WriteError, WriteGuard, Writer};
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

pub trait AsyncWriter<M: Flat + ?Sized>: CommonWriter<M> {
    type WriteFuture<'a>: Future<Output = Result<(), WriteError>> + Send + 'a
    where
        Self: 'a;
    fn write_buffer(&mut self, count: usize) -> Self::WriteFuture<'_>;
}

pub trait AsyncWriteGuard<'a> {
    type WriteFuture: Future<Output = Result<(), WriteError>>;
    fn write(self) -> Self::WriteFuture;
}

pub struct WriteFuture<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    write: Option<&'a mut W>,
    data: &'a [u8],
    poisoned: &'a mut bool,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Unpin for WriteFuture<'a, M, W> {}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Future for WriteFuture<'a, M, W> {
    type Output = Result<(), WriteError>;

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
                        return Poll::Ready(Err(WriteError::Eof));
                    }
                }
                Poll::Ready(Err(e)) => {
                    *(self.poisoned) = true;
                    return Poll::Ready(Err(WriteError::Io(e)));
                }
            }
        }
        assert!(self.write.replace(write).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> FusedFuture for WriteFuture<'a, M, W> {
    fn is_terminated(&self) -> bool {
        self.write.is_none()
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Send + Unpin> AsyncWriter<M> for Writer<M, W> {
    type WriteFuture<'a> = WriteFuture<'a, M, W> where Self: 'a;

    fn write_buffer(&mut self, count: usize) -> Self::WriteFuture<'_> {
        assert!(!self.poisoned);
        WriteFuture {
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

pub struct AsyncSharedWriter<M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    base: Arc<Base<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> AsyncSharedWriter<M, W> {
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

    pub fn alloc_message(&mut self) -> UninitWriteGuard<'_, M, Self> {
        UninitWriteGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> Clone for AsyncSharedWriter<M, W> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> CommonWriter<M> for AsyncSharedWriter<M, W> {
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

enum SharedWriteState<'a, W: AsyncWrite + Unpin> {
    Wait,
    Lock(MutexLockFuture<'a, W>),
    Write(MutexGuard<'a, W>),
}

pub struct SharedWriteFuture<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    owner: &'a AsyncSharedWriter<M, W>,
    state: Option<SharedWriteState<'a, W>>,
    data: &'a [u8],
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Unpin for SharedWriteFuture<'a, M, W> {}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> Future for SharedWriteFuture<'a, M, W> {
    type Output = Result<(), WriteError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.take().unwrap();
        let mut poll = true;
        while poll {
            (poll, state) = match state {
                SharedWriteState::Wait => (true, SharedWriteState::Lock(self.owner.base.write.lock())),
                SharedWriteState::Lock(mut lock) => match lock.poll_unpin(cx) {
                    Poll::Pending => (false, SharedWriteState::Lock(lock)),
                    Poll::Ready(writer) => {
                        assert!(!self.owner.base.poisoned.load(Ordering::Relaxed));
                        (true, SharedWriteState::Write(writer))
                    }
                },
                SharedWriteState::Write(mut writer) => match Pin::new(&mut *writer).poll_write(cx, self.data) {
                    Poll::Pending => (false, SharedWriteState::Write(writer)),
                    Poll::Ready(Ok(n)) => {
                        if n > 0 {
                            self.data = &self.data[n..];
                            if self.data.is_empty() {
                                return Poll::Ready(Ok(()));
                            } else {
                                (true, SharedWriteState::Write(writer))
                            }
                        } else {
                            return Poll::Ready(Err(WriteError::Eof));
                        }
                    }
                    Poll::Ready(Err(e)) => {
                        self.owner.base.poisoned.store(true, Ordering::Relaxed);
                        return Poll::Ready(Err(WriteError::Io(e)));
                    }
                },
            };
        }
        assert!(self.state.replace(state).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> FusedFuture for SharedWriteFuture<'a, M, W> {
    fn is_terminated(&self) -> bool {
        self.state.is_some()
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Send + Unpin> AsyncWriter<M> for AsyncSharedWriter<M, W> {
    type WriteFuture<'a> = SharedWriteFuture<'a, M, W> where Self: 'a;
    fn write_buffer(&mut self, count: usize) -> Self::WriteFuture<'_> {
        SharedWriteFuture {
            owner: self,
            state: Some(SharedWriteState::Wait),
            data: &self.buffer[..count],
        }
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> AsyncWriteGuard<'a> for WriteGuard<'a, M, O> {
    type WriteFuture = <O as AsyncWriter<M>>::WriteFuture<'a>;
    fn write(self) -> Self::WriteFuture {
        self.owner.write_buffer(self.size())
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> Unpin for UninitWriteGuard<'a, M, O> {}
impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> Unpin for WriteGuard<'a, M, O> {}
