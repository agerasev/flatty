use flatty::{utils::alloc::AlignedBytes, Flat};
use std::{io, marker::PhantomData, ops::Range};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Parse(flatty::Error),
    /// Stream has been closed.
    Eof,
}

pub struct ReadBuffer<M: Flat + ?Sized> {
    buffer: AlignedBytes,
    window: Range<usize>,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized> ReadBuffer<M> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: AlignedBytes::new(capacity, M::ALIGN),
            window: 0..0,
            _phantom: PhantomData,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn preceding_len(&self) -> usize {
        self.window.start
    }
    pub fn occupied_len(&self) -> usize {
        self.window.end - self.window.start
    }
    pub fn vacant_len(&self) -> usize {
        self.capacity() - self.window.end
    }
    pub fn extendable_len(&self) -> usize {
        self.preceding_len() + self.vacant_len()
    }

    pub fn occupied(&self) -> &[u8] {
        &self.buffer[self.window.clone()]
    }
    pub fn vacant_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[self.window.end..]
    }

    pub fn skip_occupied(&mut self, count: usize) {
        self.window.start += count;
        assert!(self.window.start <= self.window.end);
        if self.window.is_empty() {
            self.window = 0..0;
        }
    }
    pub fn take_vacant(&mut self, count: usize) {
        self.window.end += count;
        assert!(self.window.end <= self.capacity());
    }

    pub fn try_extend_vacant(&mut self) -> bool {
        if self.window.start > 0 {
            // Move data to the beginning of buffer to get free room for next data.
            self.buffer.copy_within(self.window.clone(), 0);
            self.window = 0..(self.window.end - self.window.start);
            true
        } else {
            // Message size is greater than capacity, it cannot fit the buffer.
            false
        }
    }

    pub fn message(&self) -> Result<&M, flatty::Error> {
        M::from_bytes(self.occupied())
    }

    pub fn next_message(&self) -> Option<Result<&M, ReadError>> {
        use flatty::error::{Error, ErrorKind};
        match self.message() {
            Ok(message) => Some(Ok(message)),
            Err(err) => match err {
                Error {
                    kind: ErrorKind::InsufficientSize,
                    ..
                } => {
                    if self.extendable_len() == 0 {
                        // Message cannot fit the buffer.
                        Some(Err(ReadError::Parse(Error {
                            kind: ErrorKind::InsufficientSize,
                            pos: self.occupied_len(),
                        })))
                    } else {
                        None
                    }
                }
                other_err => Some(Err(ReadError::Parse(other_err))),
            },
        }
    }
}
