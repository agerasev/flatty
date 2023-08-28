use flatty::{self, prelude::*, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait BufferSender {
    type Error;
    type Guard<'a>: BufSendGuard<Error = Self::Error>
    where
        Self: 'a;
}

pub trait BufSendGuard: DerefMut<Target = [u8]> {
    type Error;
}

pub type SendError<E> = E;

pub struct Sender<M: Flat + ?Sized, B: BufferSender> {
    pub(crate) buf_send: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: BufferSender> Sender<M, B> {
    pub fn new(buf_send: B) -> Self {
        Self {
            buf_send,
            _ghost: PhantomData,
        }
    }
}

pub struct UninitSendGuard<'a, M: Flat + ?Sized, B: BufferSender + 'a> {
    buffer: B::Guard<'a>,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: BufferSender + 'a> UninitSendGuard<'a, M, B> {
    pub(crate) fn new(buffer: B::Guard<'a>) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.buffer
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

    pub fn new_in_place(mut self, emplacer: impl Emplacer<M>) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::new_in_place(&mut self.buffer, emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, B: BufferSender + 'a> UninitSendGuard<'a, M, B> {
    pub fn default_in_place(mut self) -> Result<SendGuard<'a, M, B>, flatty::Error> {
        M::default_in_place(&mut self.buffer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

pub struct SendGuard<'a, M: Flat + ?Sized, B: BufferSender + 'a> {
    pub(crate) buffer: B::Guard<'a>,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: BufferSender + 'a> Deref for SendGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(&self.buffer) }
    }
}

impl<'a, M: Flat + ?Sized, B: BufferSender + 'a> DerefMut for SendGuard<'a, M, B> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(&mut self.buffer) }
    }
}
