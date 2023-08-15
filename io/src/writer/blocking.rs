use crate::UninitWriteGuard;

use super::{CommonWriter, WriteGuard, Writer};
use flatty::{self, prelude::*, utils::alloc::AlignedBytes};
use std::{
    io::{self, Write},
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub trait BlockingWriter<M: Flat + ?Sized>: CommonWriter<M> {
    fn write_buffer(&mut self, count: usize) -> io::Result<()>;
}

pub trait BlockingWriteGuard<'a> {
    fn write(self) -> Result<(), io::Error>;
}

impl<M: Flat + ?Sized, W: Write> BlockingWriter<M> for Writer<M, W> {
    fn write_buffer(&mut self, count: usize) -> io::Result<()> {
        assert!(!self.poisoned);
        let res = self.write.write_all(&self.buffer[..count]);
        if res.is_err() {
            self.poisoned = true;
        }
        res
    }
}

struct Base<W: Write> {
    write: Mutex<W>,
    poisoned: AtomicBool,
}

pub struct BlockingSharedWriter<M: Flat + ?Sized, W: Write> {
    base: Arc<Base<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: Write> BlockingSharedWriter<M, W> {
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

impl<M: Flat + ?Sized, W: Write> Clone for BlockingSharedWriter<M, W> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: Write> CommonWriter<M> for BlockingSharedWriter<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn poisoned(&self) -> bool {
        self.base.poisoned.load(Ordering::Relaxed) || self.base.write.is_poisoned()
    }
}

impl<M: Flat + ?Sized, W: Write> BlockingWriter<M> for BlockingSharedWriter<M, W> {
    fn write_buffer(&mut self, count: usize) -> io::Result<()> {
        let mut guard = self.base.write.lock().unwrap();
        assert!(!self.base.poisoned.load(Ordering::Relaxed));
        let res = guard.write_all(&self.buffer[..count]);
        if res.is_err() {
            self.base.poisoned.store(true, Ordering::Relaxed);
        }
        drop(guard);
        res
    }
}

impl<'a, M: Flat + ?Sized, O: BlockingWriter<M>> BlockingWriteGuard<'a> for WriteGuard<'a, M, O> {
    fn write(self) -> Result<(), io::Error> {
        self.owner.write_buffer(self.size())
    }
}
