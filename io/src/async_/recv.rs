#[cfg(feature = "io")]
use super::IoBuffer;
use super::RecvError;
use core::{
    future::Future,
    marker::PhantomData,
    mem::forget,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};
use flatty::{error::ErrorKind, Flat};
#[cfg(feature = "io")]
use futures::io::AsyncRead;

pub trait AsyncReadBuffer: Deref<Target = [u8]> + Unpin {
    type Error;

    /// Receive more bytes and put them in the buffer.
    /// Returns the number of received bytes, zero means that channel is closed.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize, Self::Error>>;

    fn read(&mut self) -> Read<'_, Self> {
        Read(self)
    }

    /// Skip first `count` bytes. Remaining bytes *may* be discarded.
    fn skip(&mut self, count: usize);
}

pub struct Read<'a, B: AsyncReadBuffer + ?Sized>(&'a mut B);

impl<B: AsyncReadBuffer + ?Sized> Future for Read<'_, B> {
    type Output = Result<usize, B::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_read(cx)
    }
}

pub struct Receiver<M: Flat + ?Sized, B: AsyncReadBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Receiver<M, B> {
    pub fn new(buffer: B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }
}

#[cfg(feature = "io")]
pub type IoReceiver<M, P> = Receiver<M, IoBuffer<P>>;

#[cfg(feature = "io")]
impl<M: Flat + ?Sized, P: AsyncRead + Unpin> IoReceiver<M, P> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN))
    }
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Receiver<M, B> {
    pub async fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        while let Err(e) = M::validate(&self.buffer) {
            match e.kind {
                ErrorKind::InsufficientSize => (),
                _ => return Err(RecvError::Parse(e)),
            }
            if self.buffer.read().await.map_err(RecvError::Read)? == 0 {
                return Err(RecvError::Closed);
            }
        }
        Ok(RecvGuard::new(&mut self.buffer))
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: AsyncReadBuffer + 'a> {
    pub(crate) buffer: &'a mut B,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: AsyncReadBuffer + 'a> RecvGuard<'a, M, B> {
    pub(crate) fn new(buffer: &'a mut B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }
    /// Destroy guard but do not remove message from receiver.
    ///
    /// Effect of this call is the same as leak of the guard.
    pub fn retain(self) {
        forget(self);
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncReadBuffer + 'a> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncReadBuffer + 'a> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}
