use core::{marker::PhantomData, mem::forget, ops::Deref};
use flatty::Flat;

pub trait BufferReceiver {
    type Error;
    type Guard<'a>: BufRecvGuard<Error = Self::Error>
    where
        Self: 'a;
}

pub trait BufRecvGuard: Deref<Target = [u8]> {
    type Error;
    fn extend(&mut self) -> Result<(), Self::Error>;
    fn skip(&mut self, count: usize);
}

#[derive(Debug)]
pub enum RecvError<E> {
    Buffer(E),
    Parse(flatty::Error),
}

pub struct Receiver<M: Flat + ?Sized, B: BufferReceiver> {
    pub(crate) buf_recv: B,
    _ghost: PhantomData<M>,
}

impl<M: Flat + ?Sized, B: BufferReceiver> Receiver<M, B> {
    pub fn new(buf_recv: B) -> Self {
        Self {
            buf_recv,
            _ghost: PhantomData,
        }
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: BufferReceiver + 'a> {
    pub(crate) buffer: B::Guard<'a>,
    _ghost: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, B: BufferReceiver + 'a> RecvGuard<'a, M, B> {
    pub(crate) fn new(buffer: B::Guard<'a>) -> Self {
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

impl<'a, M: Flat + ?Sized, B: BufferReceiver + 'a> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
    }
}

impl<'a, M: Flat + ?Sized, B: BufferReceiver + 'a> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(&self.buffer) }
    }
}
