use super::buffer::ReadBuffer;
use flatty::Flat;
use std::{io, mem::forget, ops::Deref};

#[derive(Debug)]
pub enum RecvError {
    Io(io::Error),
    Parse(flatty::Error),
    /// Stream has been closed.
    Eof,
}

pub trait CommonReceiver<M: Flat + ?Sized>: Sized {
    type RecvGuard<'a>: Sized
    where
        Self: 'a;
}

pub struct Receiver<M: Flat + ?Sized, R> {
    pub(super) reader: R,
    pub(super) buffer: ReadBuffer<M>,
}

impl<M: Flat + ?Sized, R> Receiver<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: ReadBuffer::new(max_msg_size),
        }
    }
}

impl<M: Flat + ?Sized, R> CommonReceiver<M> for Receiver<M, R> {
    type RecvGuard<'a> = RecvGuard<'a, M, R> where Self: 'a;
}

pub struct RecvGuard<'a, M: Flat + ?Sized, R> {
    owner: &'a mut Receiver<M, R>,
}

impl<'a, M: Flat + ?Sized, R> RecvGuard<'a, M, R> {
    pub(super) fn new(owner: &'a mut Receiver<M, R>) -> Self {
        Self { owner }
    }
    /// Destroy guard but do not remove message from reader.
    ///
    /// Effect of this call is the same as leak of the guard.
    pub fn retain(self) {
        forget(self);
    }
}

impl<'a, M: Flat + ?Sized, R> Drop for RecvGuard<'a, M, R> {
    fn drop(&mut self) {
        let size = self.size();
        self.owner.buffer.skip_occupied(size);
    }
}

impl<'a, M: Flat + ?Sized, R> Deref for RecvGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.owner.buffer.message().unwrap()
    }
}
