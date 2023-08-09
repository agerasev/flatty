use crate::UninitWriteGuard;

use super::{CommonWriter, WriteGuard, Writer};
use flatty::{self, prelude::*, utils::alloc::AlignedBytes};
use futures::{
    io::{AsyncWrite, AsyncWriteExt},
    lock::Mutex,
};
use std::{
    future::Future,
    io,
    marker::PhantomData,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub trait AsyncWriter<M: Flat + ?Sized>: CommonWriter<M> {
    fn write_buffer(&mut self, count: usize) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + '_>>;
}

pub trait AsyncWriteGuard<'a> {
    fn write(self) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + 'a>>;
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> AsyncWriter<M> for Writer<M, W> {
    fn write_buffer(&mut self, count: usize) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + '_>> {
        Box::pin(async move {
            assert!(!self.poisoned);
            let res = self.write.write_all(&self.buffer[..count]).await;
            if res.is_err() {
                self.poisoned = true;
            }
            res
        })
    }
}

struct Base<W: AsyncWrite + Unpin> {
    write: Mutex<W>,
    poisoned: AtomicBool,
}

pub struct AsyncSharedWriter<M: Flat + ?Sized, W: AsyncWrite + Unpin> {
    base: Arc<Base<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> AsyncSharedWriter<M, W> {
    pub fn new(write: W, max_msg_size: usize) -> Self {
        Self {
            base: Arc::new(Base {
                write: Mutex::new(write),
                poisoned: AtomicBool::new(false),
            }),
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc_message(&mut self) -> UninitWriteGuard<'_, M, Self> {
        UninitWriteGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> Clone for AsyncSharedWriter<M, W> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: AsyncWrite + Unpin> CommonWriter<M> for AsyncSharedWriter<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn poisoned(&self) -> bool {
        self.base.poisoned.load(Ordering::Relaxed)
    }
}

impl<'a, M: Flat + ?Sized, W: AsyncWrite + Unpin> AsyncWriter<M> for AsyncSharedWriter<M, W> {
    fn write_buffer(&mut self, count: usize) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + '_>> {
        Box::pin(async move {
            let mut guard = self.base.write.lock().await;
            assert!(!self.base.poisoned.load(Ordering::Relaxed));
            let res = guard.write_all(&self.buffer[..count]).await;
            if res.is_err() {
                self.base.poisoned.store(true, Ordering::Relaxed);
            }
            drop(guard);
            res
        })
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> AsyncWriteGuard<'a> for WriteGuard<'a, M, O> {
    fn write(self) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + 'a>> {
        self.owner.write_buffer(self.size())
    }
}

impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> Unpin for UninitWriteGuard<'a, M, O> {}
impl<'a, M: Flat + ?Sized, O: AsyncWriter<M>> Unpin for WriteGuard<'a, M, O> {}
