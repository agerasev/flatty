use async_std::io::{self, Read};
use flatty::{self, prelude::*};
use std::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, Range},
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub enum MsgReadError {
    Io(io::Error),
    Parse(flatty::Error),
    /// Stream has been closed.
    Eof,
}

struct MessageBuffer<M: Portable + ?Sized> {
    buffer: Vec<u8>,
    window: Range<usize>,
    _phantom: PhantomData<M>,
}

impl<M: Portable + ?Sized> MessageBuffer<M> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            window: 0..0,
            _phantom: PhantomData,
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
        M::from_bytes(self.occupied()).and_then(|m| m.validate())
    }

    fn next_message(&self) -> Option<Result<(), MsgReadError>> {
        match self.message() {
            Ok(_) => Some(Ok(())),
            Err(err) => match err {
                flatty::Error {
                    kind: flatty::ErrorKind::InsufficientSize,
                    ..
                } => {
                    if self.extendable_len() == 0 {
                        // Message cannot fit the buffer.
                        Some(Err(MsgReadError::Parse(flatty::Error {
                            kind: flatty::ErrorKind::InsufficientSize,
                            pos: self.occupied_len(),
                        })))
                    } else {
                        None
                    }
                }
                other_err => Some(Err(MsgReadError::Parse(other_err))),
            },
        }
    }
}

// Reader

pub struct AsyncReader<M: Portable + ?Sized, R: Read + Unpin> {
    reader: R,
    buffer: Option<MessageBuffer<M>>,
}

impl<M: Portable + ?Sized, R: Read + Unpin> AsyncReader<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: Some(MessageBuffer::new(max_msg_size)),
        }
    }

    fn poll_read_msg(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), MsgReadError>> {
        let mut buffer = self.buffer.take().unwrap();
        let poll = loop {
            match buffer.next_message() {
                Some(result) => break Poll::Ready(result),
                None => {
                    if buffer.vacant_len() == 0 {
                        assert!(buffer.try_extend_vacant());
                    }
                }
            }
            let reader = Pin::new(&mut self.reader);
            match reader.poll_read(cx, buffer.vacant_mut()) {
                Poll::Ready(res) => match res {
                    Ok(count) => {
                        if count != 0 {
                            buffer.take_vacant(count);
                        } else {
                            break Poll::Ready(Err(MsgReadError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(MsgReadError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        assert!(self.buffer.replace(buffer).is_none());
        poll
    }

    fn take_msg(&mut self) -> MsgReadGuard<'_, M, R> {
        MsgReadGuard { owner: self }
    }

    fn consume(&mut self, count: usize) {
        self.buffer.as_mut().unwrap().skip_occupied(count);
    }

    pub fn read_msg(&mut self) -> MsgReadFuture<'_, M, R> {
        MsgReadFuture { owner: Some(self) }
    }
}

impl<M: Portable + ?Sized, R: Read + Unpin> Unpin for AsyncReader<M, R> {}

// ReadGuard

pub struct MsgReadGuard<'a, M: Portable + ?Sized, R: Read + Unpin> {
    owner: &'a mut AsyncReader<M, R>,
}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Drop for MsgReadGuard<'a, M, R> {
    fn drop(&mut self) {
        self.owner.consume(self.size());
    }
}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Deref for MsgReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.owner.buffer.as_ref().unwrap().message().unwrap()
    }
}

// ReadFuture

pub struct MsgReadFuture<'a, M: Portable + ?Sized, R: Read + Unpin> {
    owner: Option<&'a mut AsyncReader<M, R>>,
}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Unpin for MsgReadFuture<'a, M, R> {}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Future for MsgReadFuture<'a, M, R> {
    type Output = Result<MsgReadGuard<'a, M, R>, MsgReadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self.owner.take().unwrap();
        match owner.poll_read_msg(cx) {
            Poll::Ready(res) => Poll::Ready(res.map(|()| owner.take_msg())),
            Poll::Pending => {
                self.owner.replace(owner);
                Poll::Pending
            }
        }
    }
}
