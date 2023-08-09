use flatty::{self, prelude::*, utils::alloc::AlignedBytes, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait CommonWriter {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];

    fn poisoned(&self) -> bool;
}

pub struct Writer<M: Flat + ?Sized, W> {
    pub(super) write: W,
    pub(super) poisoned: bool,
    pub(super) buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W> Writer<M, W> {
    pub fn new(write: W, max_msg_size: usize) -> Self {
        Self {
            write,
            poisoned: false,
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc_message(&mut self) -> UninitWriteGuard<'_, M, Self> {
        UninitWriteGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W> CommonWriter for Writer<M, W> {
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

pub struct UninitWriteGuard<'a, M: Flat + ?Sized, O: CommonWriter> {
    owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonWriter> UninitWriteGuard<'a, M, O> {
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
    pub unsafe fn assume_valid(self) -> WriteGuard<'a, M, O> {
        WriteGuard {
            owner: self.owner,
            _phantom: PhantomData,
        }
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<WriteGuard<'a, M, O>, flatty::Error> {
        M::new_in_place(self.owner.buffer_mut(), emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, O: CommonWriter> UninitWriteGuard<'a, M, O> {
    pub fn default_in_place(self) -> Result<WriteGuard<'a, M, O>, flatty::Error> {
        M::default_in_place(self.owner.buffer_mut())?;
        Ok(unsafe { self.assume_valid() })
    }
}

pub struct WriteGuard<'a, M: Flat + ?Sized, O: CommonWriter> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonWriter> Deref for WriteGuard<'a, M, O> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.owner.buffer()) }
    }
}

impl<'a, M: Flat + ?Sized, O: CommonWriter> DerefMut for WriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.owner.buffer_mut()) }
    }
}
