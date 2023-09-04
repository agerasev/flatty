use core::ops::Deref;

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
