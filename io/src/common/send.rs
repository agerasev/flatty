use flatty::{self, prelude::*, utils::alloc::AlignedBytes, Emplacer};
use std::{
    io,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub enum SendError {
    Io(io::Error),
    /// Stream has been closed.
    Eof,
}

pub trait CommonSender<M: Flat + ?Sized>: Sized {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];

    fn poisoned(&self) -> bool;
}

pub struct Sender<M: Flat + ?Sized, W> {
    pub(crate) write: W,
    pub(crate) poisoned: bool,
    pub(crate) buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W> Sender<M, W> {
    pub fn new(write: W, max_msg_size: usize) -> Self {
        Self {
            write,
            poisoned: false,
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc(&mut self) -> UninitSendGuard<'_, M, Self> {
        UninitSendGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W> CommonSender<M> for Sender<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn poisoned(&self) -> bool {
        self.poisoned
    }
}

pub struct UninitSendGuard<'a, M: Flat + ?Sized, O: CommonSender<M>> {
    owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonSender<M>> UninitSendGuard<'a, M, O> {
    pub fn new(owner: &'a mut O) -> Self {
        Self {
            owner,
            _phantom: PhantomData,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        self.owner.buffer()
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.owner.buffer_mut()
    }

    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_valid(self) -> SendGuard<'a, M, O> {
        SendGuard {
            owner: self.owner,
            _phantom: PhantomData,
        }
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<SendGuard<'a, M, O>, flatty::Error> {
        M::new_in_place(self.owner.buffer_mut(), emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, O: CommonSender<M>> UninitSendGuard<'a, M, O> {
    pub fn default_in_place(self) -> Result<SendGuard<'a, M, O>, flatty::Error> {
        M::default_in_place(self.owner.buffer_mut())?;
        Ok(unsafe { self.assume_valid() })
    }
}

pub struct SendGuard<'a, M: Flat + ?Sized, O: CommonSender<M>> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonSender<M>> Deref for SendGuard<'a, M, O> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.owner.buffer()) }
    }
}

impl<'a, M: Flat + ?Sized, O: CommonSender<M>> DerefMut for SendGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.owner.buffer_mut()) }
    }
}
