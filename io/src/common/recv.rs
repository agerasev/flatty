use core::{marker::PhantomData, mem::forget, ops::Deref};
use flatty::Flat;

pub trait ReadBuffer: Deref<Target = [u8]> {
    type Error;
    /// Skip first `count` bytes. Remaining bytes *may* be discarded.
    fn skip(&mut self, count: usize);
}

#[derive(Debug)]
pub enum RecvError<E> {
    Buffer(E),
    Parse(flatty::Error),
    Closed,
}

pub struct Receiver<M: Flat + ?Sized, B: ReadBuffer> {
    pub(crate) buffer: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: ReadBuffer> Receiver<M, B> {
    pub fn new(buffer: B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> {
    pub(crate) buffer: &'a mut B,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> RecvGuard<'a, M, B> {
    pub(crate) fn new(buffer: &'a mut B) -> Self {
        Self {
            buffer,
            _ghost: PhantomData,
        }
    }
    /// Destroy guard but do not remove message from receiver.
    ///
    /// Effect of this call is the same as leak of the guard.
    pub fn retain(self) {
        forget(self);
    }
}

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
    }
}

impl<'a, M: Flat + ?Sized, B: ReadBuffer + 'a> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.buffer) }
    }
}
