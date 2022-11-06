use super::{CommonUninitWriteGuard, CommonWriteGuard, CommonWriter};
use flatty::{self, prelude::*};
use std::{
    io::{self, Write},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub struct Writer<M: Portable + ?Sized, W: Write> {
    writer: Arc<Mutex<W>>,
    buffer: Vec<u8>,
    _phantom: PhantomData<M>,
}

impl<M: Portable + ?Sized, W: Write> Writer<M, W> {
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

impl<M: Portable + ?Sized, W: Write> Clone for Writer<M, W> {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
            buffer: vec![0; self.buffer.len()],
            _phantom: PhantomData,
        }
    }
}

impl<M: Portable + ?Sized, W: Write> CommonWriter<M> for Writer<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

pub type UninitWriteGuard<'a, M, W> = CommonUninitWriteGuard<'a, M, Writer<M, W>>;

pub type WriteGuard<'a, M, W> = CommonWriteGuard<'a, M, Writer<M, W>>;

impl<'a, M: Portable + ?Sized, W: Write> WriteGuard<'a, M, W> {
    pub fn write(self) -> Result<(), io::Error> {
        let mut guard = self.owner.writer.lock().unwrap();
        guard.write_all(&self.owner.buffer[..self.size()])
    }
}
