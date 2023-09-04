#[cfg(feature = "io")]
use super::IoBuffer;
use super::{ReadBuffer, RecvError};
use core::{marker::PhantomData, mem::forget, ops::Deref};
use flatty::{error::ErrorKind, Flat};
#[cfg(feature = "io")]
use std::io::Read;

pub trait BlockingReadBuffer: ReadBuffer {
    /// Receive more bytes and put them in the buffer.
    /// Returns the number of received bytes, zero means that channel is closed.
    fn read(&mut self) -> Result<usize, Self::Error>;
}

pub struct Receiver<M: Flat + ?Sized, B: ReadBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: ReadBuffer> Receiver<M, B> {
    pub fn new(buffer: B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }
}

#[cfg(feature = "io")]
impl<M: Flat + ?Sized, P: Read> Receiver<M, IoBuffer<P>> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN))
    }
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> Receiver<M, B> {
    pub fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        while let Err(e) = M::validate(&self.buffer) {
            match e.kind {
                ErrorKind::InsufficientSize => (),
                _ => return Err(RecvError::Parse(e)),
            }
            if self.buffer.read().map_err(RecvError::Buffer)? == 0 {
                return Err(RecvError::Closed);
            }
        }
        Ok(RecvGuard::new(&mut self.buffer))
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> {
    pub(crate) buffer: &'a mut B,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> RecvGuard<'a, M, B> {
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

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
    }
}

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}
