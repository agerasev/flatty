use super::{CommonUninitWriteGuard, CommonWriteGuard, CommonWriter};
use flatty::{self, prelude::*};
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

    pub fn new_message(&mut self) -> UninitWriteGuard<'_, M, W> {
        UninitWriteGuard::new(self)
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

pub type UninitWriteGuard<'a, M, W> = CommonUninitWriteGuard<'a, M, AsyncWriter<M, W>>;

pub type WriteGuard<'a, M, W> = CommonWriteGuard<'a, M, AsyncWriter<M, W>>;

impl<'a, M: Portable + ?Sized, W: AsyncWrite + Unpin> WriteGuard<'a, M, W> {
    pub async fn write_async(self) -> Result<(), io::Error> {
        let mut guard = self.owner.writer.lock().await;
        guard.write_all(&self.owner.buffer[..self.size()]).await
    }
}
