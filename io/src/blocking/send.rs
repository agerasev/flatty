use crate::common::{CommonSender, SendError, SendGuard, Sender};
use flatty::{self, prelude::*};
use std::io::Write;

pub trait BlockingSender<M: Flat + ?Sized>: CommonSender<M> {
    fn send_buffer(&mut self, count: usize) -> Result<(), SendError>;
}

pub trait BlockingSendGuard<'a> {
    fn send(self) -> Result<(), SendError>;
}

impl<M: Flat + ?Sized, W: Write> BlockingSender<M> for Sender<M, W> {
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

impl<'a, M: Flat + ?Sized, O: BlockingSender<M>> BlockingSendGuard<'a> for SendGuard<'a, M, O> {
    fn send(self) -> Result<(), SendError> {
        self.owner.send_buffer(self.size())
    }
}
