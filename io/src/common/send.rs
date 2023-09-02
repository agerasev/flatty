use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use flatty::{self, prelude::*, Emplacer};

pub trait WriteBuffer: DerefMut<Target = [u8]> {
    type Error;
}

pub type SendError<E> = E;

pub struct Sender<M: Flat + ?Sized, B: WriteBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: WriteBuffer> Sender<M, B> {
    pub fn new(buf_send: B) -> Self {
        Self {
            buffer: buf_send,
            _ghost: PhantomData,
        }
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
