#[cfg(feature = "io")]
use super::IoBuffer;
use super::{SendError, WriteBuffer};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use flatty::{self, prelude::*, Emplacer};
#[cfg(feature = "io")]
use std::io::Write;

pub trait BlockingWriteBuffer: WriteBuffer {
    /// Allocate some fixed amount of bytes in the buffer.
    fn alloc(&mut self) -> Result<(), Self::Error>;
    /// Send exactly `count` bytes from buffer.
    /// Remaining bytes are discarded.
    fn write_all(&mut self, count: usize) -> Result<(), Self::Error>;
}

pub struct Sender<M: Flat + ?Sized, B: BlockingWriteBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: BlockingWriteBuffer> Sender<M, B> {
    pub fn new(buf_send: B) -> Self {
        Self {
            buffer: buf_send,
            _ghost: PhantomData,
        }
    }
}

#[cfg(feature = "io")]
impl<M: Flat + ?Sized, P: Write> Sender<M, IoBuffer<P>> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN))
    }
}

impl<M: Flat + ?Sized, B: BlockingWriteBuffer> Sender<M, B> {
    pub fn alloc(&mut self) -> Result<UninitSendGuard<'_, M, B>, SendError<B::Error>> {
        self.buffer.alloc()?;
        Ok(UninitSendGuard::new(&mut self.buffer))
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer> SendGuard<'a, M, B> {
    pub fn send(self) -> Result<(), SendError<B::Error>> {
        let size = self.size();
        self.buffer.write_all(size)
    }
}

pub struct SendGuard<'a, M: Flat + ?Sized, B: WriteBuffer + 'a, const INIT: bool = true> {
    pub(crate) buffer: &'a mut B,
    _ghost: PhantomData<M>,
}

pub type UninitSendGuard<'a, M, B> = SendGuard<'a, M, B, false>;

impl<'a, M: Flat + ?Sized, B: WriteBuffer + 'a> UninitSendGuard<'a, M, B> {
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
    pub unsafe fn assume_valid(self) -> SendGuard<'a, M, B> {
        SendGuard {
            buffer: self.buffer,
            _ghost: PhantomData,
        }
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::new_in_place(self.buffer, emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, B: WriteBuffer + 'a> UninitSendGuard<'a, M, B> {
    pub fn default_in_place(self) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::default_in_place(self.buffer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + ?Sized, B: WriteBuffer + 'a> Deref for SendGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}

impl<'a, M: Flat + ?Sized, B: WriteBuffer + 'a> DerefMut for SendGuard<'a, M, B> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.buffer) }
    }
}
