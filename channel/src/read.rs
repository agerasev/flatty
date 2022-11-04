use async_std::io::{self, Read};
use flatty::{self, prelude::*};
use std::{
    future::Future,
    marker::PhantomData,
    mem,
    ops::{Deref, Range},
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub enum MsgReadError {
    Io(io::Error),
    Parse(flatty::Error),
    Eof,
}

// Reader

pub struct MsgReader<M: Portable + ?Sized, R: Read + Unpin> {
    reader: R,
    buffer: Vec<u8>,
    window: Range<usize>,
    _phantom: PhantomData<M>,
}

impl<M: Portable + ?Sized, R: Read + Unpin> MsgReader<M, R> {
    pub fn new(reader: R, max_msg_size: usize) -> Self {
        Self {
            reader,
            buffer: vec![0; max_msg_size],
            window: 0..0,
            _phantom: PhantomData,
        }
    }

    fn poll_read_msg(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), MsgReadError>> {
        let mut buffer = Vec::new();
        mem::swap(&mut buffer, &mut self.buffer);
        let mut window = self.window.clone();
        let result = loop {
            match M::from_bytes(&buffer[window.clone()]).and_then(|m| m.validate()) {
                Ok(_) => break Poll::Ready(Ok(())),
                Err(err) => match err {
                    flatty::Error {
                        kind: flatty::ErrorKind::InsufficientSize,
                        ..
                    } => {
                        if window.end == buffer.len() {
                            // No free space at the end of the buffer.
                            if window.start != 0 {
                                // Move data to the beginning of buffer to get free room for next data.
                                buffer.copy_within(window.clone(), 0);
                                window = 0..(window.end - window.start);
                            } else {
                                // Message is greater than `max_msg_size`, it cannot fit the buffer.
                                break Poll::Ready(Err(MsgReadError::Parse(flatty::Error {
                                    kind: flatty::ErrorKind::InsufficientSize,
                                    pos: buffer.len(),
                                })));
                            }
                        }
                    }
                    other_err => break Poll::Ready(Err(MsgReadError::Parse(other_err))),
                },
            };
            let reader = Pin::new(&mut self.reader);
            match reader.poll_read(cx, &mut buffer[window.end..]) {
                Poll::Ready(res) => match res {
                    Ok(count) => {
                        if count != 0 {
                            window.end += count;
                            assert!(window.end <= buffer.len());
                        } else {
                            break Poll::Ready(Err(MsgReadError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(MsgReadError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        mem::swap(&mut buffer, &mut self.buffer);
        self.window = window;
        result
    }

    fn take_msg(&mut self) -> MsgReadGuard<'_, M, R> {
        MsgReadGuard { owner: self }
    }

    fn consume(&mut self, count: usize) {
        self.window.start += count;
        assert!(self.window.start <= self.window.end);
        if self.window.is_empty() {
            self.window = 0..0;
        }
    }

    pub fn read_msg(&mut self) -> MsgReadFuture<'_, M, R> {
        MsgReadFuture { owner: Some(self) }
    }
}

impl<M: Portable + ?Sized, R: Read + Unpin> Unpin for MsgReader<M, R> {}

// ReadGuard

pub struct MsgReadGuard<'a, M: Portable + ?Sized, R: Read + Unpin> {
    owner: &'a mut MsgReader<M, R>,
}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Drop for MsgReadGuard<'a, M, R> {
    fn drop(&mut self) {
        self.owner.consume(self.size());
    }
}

impl<'a, M: Portable + ?Sized, R: Read + Unpin> Deref for MsgReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        M::from_bytes(&self.owner.buffer[self.owner.window.clone()])
            .unwrap()
            .validate()
            .unwrap()
    }
}

// ReadFuture

pub struct MsgReadFuture<'a, M: Portable + ?Sized, R: Read + Unpin> {
    owner: Option<&'a mut MsgReader<M, R>>,
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
