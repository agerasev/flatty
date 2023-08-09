use super::buffer::ReadBuffer;
use flatty::Flat;
use std::{mem::forget, ops::Deref};

pub trait CommonReader<M: Flat + ?Sized>: Sized {
    type ReadGuard<'a>: Sized
    where
        Self: 'a;
}

pub struct Reader<M: Flat + ?Sized, R> {
    pub(super) reader: R,
    pub(super) buffer: ReadBuffer<M>,
}

impl<M: Flat + ?Sized, R> Reader<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: ReadBuffer::new(max_msg_size),
        }
    }
}

impl<M: Flat + ?Sized, R> CommonReader<M> for Reader<M, R> {
    type ReadGuard<'a> = ReadGuard<'a, M, R> where Self: 'a;
}

pub struct ReadGuard<'a, M: Flat + ?Sized, R> {
    owner: &'a mut Reader<M, R>,
}

impl<'a, M: Flat + ?Sized, R> ReadGuard<'a, M, R> {
    pub(super) fn new(owner: &'a mut Reader<M, R>) -> Self {
        Self { owner }
    }
    /// Destroy guard but do not remove message from reader.
    ///
    /// Effect of this call is the same as leak of the guard.
    pub fn retain(self) {
        forget(self);
    }
}

impl<'a, M: Flat + ?Sized, R> Drop for ReadGuard<'a, M, R> {
    fn drop(&mut self) {
        let size = self.size();
        self.owner.buffer.skip_occupied(size);
    }
}

impl<'a, M: Flat + ?Sized, R> Deref for ReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.owner.buffer.message().unwrap()
    }
}
