use super::{CommonUninitWriteGuard, CommonWriteGuard, CommonWriter};
use derive_more::*;
use flatty::{self, prelude::*, Emplacer};
use futures::{
    io::{AsyncWrite, AsyncWriteExt},
    lock::Mutex,
};
use std::{io, marker::PhantomData, sync::Arc};

pub struct AsyncWriter<M: Portable + ?Sized, W: AsyncWrite + Unpin> {
    writer: Arc<Mutex<W>>,
    buffer: Vec<u8>,
    _phantom: PhantomData<M>,
}

impl<M: Portable + ?Sized, W: AsyncWrite + Unpin> AsyncWriter<M, W> {
    pub fn new(writer: W, max_msg_size: usize) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
            buffer: vec![0; max_msg_size],
            _phantom: PhantomData,
        }
    }

    pub fn alloc_message(&mut self) -> AsyncUninitWriteGuard<'_, M, W> {
        CommonUninitWriteGuard::new(self).into()
    }
}

impl<M: Portable + ?Sized, W: AsyncWrite + Unpin> Clone for AsyncWriter<M, W> {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
            buffer: vec![0; self.buffer.len()],
            _phantom: PhantomData,
        }
    }
}

impl<M: Portable + ?Sized, W: AsyncWrite + Unpin> CommonWriter<M> for AsyncWriter<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

#[derive(From, Into, Deref, DerefMut)]
pub struct AsyncUninitWriteGuard<'a, M: Portable + ?Sized, W: AsyncWrite + Unpin> {
    inner: CommonUninitWriteGuard<'a, M, AsyncWriter<M, W>>,
}

impl<'a, M: Portable + ?Sized, W: AsyncWrite + Unpin> AsyncUninitWriteGuard<'a, M, W> {
    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_valid(self) -> AsyncWriteGuard<'a, M, W> {
        CommonUninitWriteGuard::from(self).assume_valid().into()
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<AsyncWriteGuard<'a, M, W>, flatty::Error> {
        CommonUninitWriteGuard::from(self)
            .new_in_place(emplacer)
            .map(|common| common.into())
    }
}

impl<'a, M: Portable + FlatDefault + ?Sized, W: AsyncWrite + Unpin> AsyncUninitWriteGuard<'a, M, W> {
    pub fn default(self) -> Result<AsyncWriteGuard<'a, M, W>, flatty::Error> {
        CommonUninitWriteGuard::from(self)
            .default_in_place()
            .map(|common| common.into())
    }
}

#[derive(From, Into, Deref, DerefMut)]
pub struct AsyncWriteGuard<'a, M: Portable + ?Sized, W: AsyncWrite + Unpin> {
    inner: CommonWriteGuard<'a, M, AsyncWriter<M, W>>,
}

impl<'a, M: Portable + ?Sized, W: AsyncWrite + Unpin> AsyncWriteGuard<'a, M, W> {
    pub async fn write(self) -> Result<(), io::Error> {
        let mut guard = self.owner.writer.lock().await;
        guard.write_all(&self.owner.buffer[..self.size()]).await
    }
}
