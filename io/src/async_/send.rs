use super::SendError;
#[cfg(feature = "io")]
use crate::IoBuffer;
use core::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};
use flatty::{self, prelude::*, Emplacer};
#[cfg(feature = "io")]
use futures::io::AsyncWrite;

pub trait AsyncWriteBuffer: DerefMut<Target = [u8]> + Unpin {
    type Error;

    /// Allocate some fixed amount of bytes in the buffer.
    fn poll_alloc(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Allocate some fixed amount of bytes in the buffer.
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

pub struct Sender<M: Flat + ?Sized, B: AsyncWriteBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: AsyncWriteBuffer> Sender<M, B> {
    pub fn new(buf_send: B) -> Self {
        Self {
            buffer: buf_send,
            _ghost: PhantomData,
        }
    }
}

#[cfg(feature = "io")]
pub type IoSender<M, P> = Sender<M, IoBuffer<P>>;

#[cfg(feature = "io")]
impl<M: Flat + ?Sized, P: AsyncWrite + Unpin> IoSender<M, P> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN))
    }
}

impl<M: Flat + ?Sized, B: AsyncWriteBuffer> Sender<M, B> {
    pub async fn alloc(&mut self) -> Result<UninitSendGuard<'_, M, B>, SendError<B::Error>> {
        self.buffer.alloc().await?;
        Ok(UninitSendGuard::new(&mut self.buffer))
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer> SendGuard<'a, M, B> {
    pub fn send(self) -> B::WriteAllFuture<'a> {
        let size = self.size();
        self.buffer.write_all(size)
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer, const INIT: bool> Unpin for SendGuard<'a, M, B, INIT> {}

pub struct SendGuard<'a, M: Flat + ?Sized, B: AsyncWriteBuffer + 'a, const INIT: bool = true> {
    pub(crate) buffer: &'a mut B,
    _ghost: PhantomData<M>,
}

pub type UninitSendGuard<'a, M, B> = SendGuard<'a, M, B, false>;

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer + 'a> UninitSendGuard<'a, M, B> {
    pub(crate) fn new(buffer: &'a mut B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buffer
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.buffer
    }

    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_init(self) -> SendGuard<'a, M, B> {
        SendGuard {
            buffer: self.buffer,
            _ghost: PhantomData,
        }
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::new_in_place(self.buffer, emplacer)?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, B: AsyncWriteBuffer + 'a> UninitSendGuard<'a, M, B> {
    pub fn default_in_place(self) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::default_in_place(self.buffer)?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer + 'a> Deref for SendGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncWriteBuffer + 'a> DerefMut for SendGuard<'a, M, B> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.buffer) }
    }
}
