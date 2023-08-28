use super::super::{BlockingSender, CommonSender, SendError, UninitSendGuard};
use flatty::{self, prelude::*, utils::alloc::AlignedBytes};
use std::{
    io::Write,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

struct Base<W: Write> {
    write: Mutex<W>,
    poisoned: AtomicBool,
}

pub struct SharedSender<M: Flat + ?Sized, W: Write> {
    base: Arc<Base<W>>,
    buffer: AlignedBytes,
    _phantom: PhantomData<M>,
}

impl<M: Flat + ?Sized, W: Write> SharedSender<M, W> {
    pub fn new(write: W, max_msg_size: usize) -> Self {
        Self {
            base: Arc::new(Base {
                write: Mutex::new(write),
                poisoned: AtomicBool::new(false),
            }),
            buffer: AlignedBytes::new(max_msg_size, M::ALIGN),
            _phantom: PhantomData,
        }
    }

    pub fn alloc_message(&mut self) -> UninitSendGuard<'_, M, Self> {
        UninitSendGuard::new(self)
    }
}

impl<M: Flat + ?Sized, W: Write> Clone for SharedSender<M, W> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            buffer: AlignedBytes::new(self.buffer.len(), M::ALIGN),
            _phantom: PhantomData,
        }
    }
}

impl<M: Flat + ?Sized, W: Write> CommonSender<M> for SharedSender<M, W> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn poisoned(&self) -> bool {
        self.base.poisoned.load(Ordering::Relaxed) || self.base.write.is_poisoned()
    }
}

impl<M: Flat + ?Sized, W: Write> BlockingSender<M> for SharedSender<M, W> {
    fn send_buffer(&mut self, count: usize) -> Result<(), SendError> {
        let mut guard = self.base.write.lock().unwrap();
        assert!(!self.base.poisoned.load(Ordering::Relaxed));

        let mut data = &self.buffer[..count];
        loop {
            match guard.write(data) {
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
                    self.base.poisoned.store(true, Ordering::Relaxed);
                    break Err(SendError::Io(e));
                }
            }
        }
    }
}
