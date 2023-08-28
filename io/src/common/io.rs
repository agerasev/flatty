use super::BufferReceiver;
use flatty::utils::alloc::AlignedBytes;
use std::{io, ops::Range};

pub struct IoSender<W> {
    pub(crate) write: W,
    pub(crate) poisoned: bool,
}

pub struct IoReceiver<R> {
    pub(crate) read: R,
    pub(crate) poisoned: bool,
    buffer: AlignedBytes,
    window: Range<usize>,
}

impl<W> BufferSender for IoSender<W> {
    type Error = io::Error;
}
impl<W> BufferReceiver for IoReceiver<W> {
    type Error = io::Error;
}

impl<R> IoReceiver<R> {
    fn new(read: R, capacity: usize, align: usize) -> Self {
        Self {
            read,
            poisoned: false,
            buffer: AlignedBytes::new(capacity, align),
            window: 0..0,
        }
    }

    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    fn preceding_len(&self) -> usize {
        self.window.start
    }
    fn occupied_len(&self) -> usize {
        self.window.end - self.window.start
    }
    fn vacant_len(&self) -> usize {
        self.capacity() - self.window.end
    }
    fn extendable_len(&self) -> usize {
        self.preceding_len() + self.vacant_len()
    }

    fn occupied(&self) -> &[u8] {
        &self.buffer[self.window.clone()]
    }
    fn vacant_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[self.window.end..]
    }

    fn skip_occupied(&mut self, count: usize) {
        self.window.start += count;
        assert!(self.window.start <= self.window.end);
        if self.window.is_empty() {
            self.window = 0..0;
        }
    }
    fn take_vacant(&mut self, count: usize) {
        self.window.end += count;
        assert!(self.window.end <= self.capacity());
    }

    fn try_extend_vacant(&mut self) -> bool {
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

    fn message(&self) -> Result<&M, flatty::Error> {
        M::from_bytes(self.occupied())
    }

    fn next_message(&self) -> Option<Result<&M, RecvError>> {
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
                        Some(Err(RecvError::Parse(Error {
                            kind: ErrorKind::InsufficientSize,
                            pos: self.occupied_len(),
                        })))
                    } else {
                        None
                    }
                }
                other_err => Some(Err(RecvError::Parse(other_err))),
            },
        }
    }

    fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        loop {
            match self.buffer.next_message() {
                Some(result) => break result.map(|_| ()),
                None => {
                    if self.buffer.vacant_len() == 0 {
                        assert!(self.buffer.try_extend_vacant());
                    }
                }
            }
            match self.reader.read(self.buffer.vacant_mut()) {
                Ok(count) => {
                    if count != 0 {
                        self.buffer.take_vacant(count);
                    } else {
                        break Err(RecvError::Eof);
                    }
                }
                Err(err) => break Err(RecvError::Io(err)),
            }
        }
        .map(|()| RecvGuard::new(self))
    }
}

impl<W> IoSender<W> {
    fn send_buffer(&mut self, count: usize) -> Result<(), SendError> {
        assert!(!self.poisoned);
        let mut data = &self.buffer[..count];
        loop {
            match self.write.write(data) {
                Ok(n) => {
                    if n > 0 {
                        data = &data[n..];
                        if data.is_empty() {
                            break Ok(());
                        }
                    } else {
                        break Err(SendError::Eof);
                    }
                }
                Err(e) => {
                    self.poisoned = true;
                    break Err(SendError::Io(e));
                }
            }
        }
    }
}
