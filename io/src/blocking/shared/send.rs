#[cfg(feature = "io")]
use super::super::IoBuffer;
use super::super::{BlockingWriteBuffer, SendError};
use flatty::{self, prelude::*, utils::alloc::AlignedBytes, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub struct SharedSender<M: Flat + ?Sized, B: BlockingWriteBuffer> {
    shared: Arc<Mutex<B>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: BlockingWriteBuffer> Clone for SharedSender<M, B> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, B: BlockingWriteBuffer> SharedSender<M, B> {
    pub fn new(buffer: B, max_msg_size: usize) -> Self {
        Self {
            shared: Arc::new(Mutex::new(buffer)),
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc(&mut self) -> Result<SendGuard<'_, M, B, false>, SendError<B::Error>> {
        Ok(SendGuard {
            shared: &self.shared,
            buffer: &mut self.buffer,
            _ghost: PhantomData,
        })
    }
}

pub struct SendGuard<'a, M: Flat + ?Sized, B: BlockingWriteBuffer + 'a, const INIT: bool = true> {
    shared: &'a Mutex<B>,
    buffer: &'a mut AlignedBytes,
    _ghost: PhantomData<M>,
}

pub type UninitSendGuard<'a, M, B> = SendGuard<'a, M, B, false>;

impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer + 'a> UninitSendGuard<'a, M, B> {
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
            shared: self.shared,
            buffer: self.buffer,
            _ghost: PhantomData,
        }
    }
    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::new_in_place(self.buffer, emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}
impl<'a, M: Flat + FlatDefault + ?Sized, B: BlockingWriteBuffer + 'a> UninitSendGuard<'a, M, B> {
    pub fn default_in_place(self) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::default_in_place(self.buffer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer + 'a> Deref for SendGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}
impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer + 'a> DerefMut for SendGuard<'a, M, B> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.buffer) }
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingWriteBuffer + 'a> SendGuard<'a, M, B> {
    pub fn send(self) -> Result<(), SendError<B::Error>> {
        let mut shared = self.shared.lock().unwrap();
        shared.alloc()?;
        let size = self.size();
        let src = &self.as_bytes()[..size];
        let dst = shared.get_mut(..size).unwrap();
        dst.copy_from_slice(src);
        shared.write_all(size)
    }
}

#[cfg(feature = "io")]
impl<M: Flat + ?Sized, P: std::io::Write> SharedSender<M, IoBuffer<P>> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN), max_msg_len)
    }
}
