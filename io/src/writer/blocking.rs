use super::{CommonUninitWriteGuard, CommonWriteGuard, CommonWriter};
use derive_more::*;
use flatty::{self, prelude::*, utils::alloc::AlignedBytes, Emplacer};
use std::{
    io::{self, Write},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub struct Writer<M: Flat + ?Sized, W: Write> {
    writer: Arc<Mutex<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: Write> Writer<M, W> {
    pub fn new(writer: W, max_msg_size: usize) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc_message(&mut self) -> UninitWriteGuard<'_, M, W> {
        CommonUninitWriteGuard::new(self).into()
    }
}

impl<M: Flat + ?Sized, W: Write> Clone for Writer<M, W> {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: Write> CommonWriter<M> for Writer<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

#[derive(From, Into, Deref, DerefMut)]
pub struct UninitWriteGuard<'a, M: Flat + ?Sized, W: Write> {
    inner: CommonUninitWriteGuard<'a, M, Writer<M, W>>,
}

impl<'a, M: Flat + ?Sized, W: Write> UninitWriteGuard<'a, M, W> {
    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_valid(self) -> WriteGuard<'a, M, W> {
        CommonUninitWriteGuard::from(self).assume_valid().into()
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<WriteGuard<'a, M, W>, flatty::Error> {
        CommonUninitWriteGuard::from(self)
            .new_in_place(emplacer)
            .map(|common| common.into())
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, W: Write> UninitWriteGuard<'a, M, W> {
    pub fn default(self) -> Result<WriteGuard<'a, M, W>, flatty::Error> {
        CommonUninitWriteGuard::from(self)
            .default_in_place()
            .map(|common| common.into())
    }
}

#[derive(From, Into, Deref, DerefMut)]
pub struct WriteGuard<'a, M: Flat + ?Sized, W: Write> {
    inner: CommonWriteGuard<'a, M, Writer<M, W>>,
}

impl<'a, M: Flat + ?Sized, W: Write> WriteGuard<'a, M, W> {
    pub fn write(self) -> Result<(), io::Error> {
        let mut guard = self.owner.writer.lock().unwrap();
        guard.write_all(&self.owner.buffer[..self.size()])
    }
}
