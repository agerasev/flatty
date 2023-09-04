#[cfg(feature = "io")]
use super::super::IoBuffer;
use super::{
    super::{BlockingReadBuffer, RecvError},
    Endpoint, EndpointTable, EptHandle, EptId, Filter,
};
use flatty::{error::ErrorKind, Flat};
#[cfg(feature = "io")]
use std::io::Read;
use std::{
    ops::Deref,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};

impl EptHandle for Arc<Condvar> {
    fn wake(&self) {
        self.notify_one();
    }
}

struct SharedData<M: Flat + ?Sized, B: BlockingReadBuffer> {
    buffer: Mutex<B>,
    table: EndpointTable<M, Arc<Condvar>>,
}

pub struct SharedReceiver<M: Flat + ?Sized, B: BlockingReadBuffer> {
    shared: Arc<SharedData<M, B>>,
    filter: Filter<M>,
    handle: Arc<Condvar>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, B: BlockingReadBuffer> SharedReceiver<M, B> {
    pub fn new(buffer: B, max_msg_size: usize) -> Self {
        let table = EndpointTable::default();
        let filter = Filter::default();
        let handle = Arc::new(Condvar::new());
        let ept = Endpoint {
            filter: filter.clone(),
            handle: handle.clone(),
        };
        let id = table.insert(ept);
        Self {
            shared: Arc::new(SharedData {
                buffer: Mutex::new(buffer),
                table,
            }),
            filter,
            handle,
            id,
        }
    }

    pub fn filter<F: Fn(&M) -> bool + Sync + Send + 'static>(mut self, f: F) -> Self {
        let mut ept = self.shared.table.get(self.id).unwrap();
        let filter = Filter::new({
            let g = ept.filter.clone();
            move |m| g.check(m) && f(m)
        });
        ept.filter = filter.clone();
        drop(ept);
        self.filter = filter;
        self
    }
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> Clone for SharedReceiver<M, B> {
    fn clone(&self) -> Self {
        let filter = self.shared.table.get(self.id).unwrap().filter.clone();
        let handle = Arc::new(Condvar::new());
        let ept = Endpoint {
            filter: filter.clone(),
            handle: handle.clone(),
        };
        let id = self.shared.table.insert(ept);
        Self {
            shared: self.shared.clone(),
            filter,
            handle,
            id,
        }
    }
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> Drop for SharedReceiver<M, B> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

#[cfg(feature = "io")]
impl<M: Flat + ?Sized + 'static, P: Read> SharedReceiver<M, IoBuffer<P>> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN), max_msg_len)
    }
}

impl<M: Flat + ?Sized, B: BlockingReadBuffer> SharedReceiver<M, B> {
    pub fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        let mut buffer = self.shared.buffer.lock().unwrap();
        loop {
            let msg = match M::from_bytes(&buffer) {
                Ok(msg) => msg,
                Err(e) => match e.kind {
                    ErrorKind::InsufficientSize => {
                        if buffer.read().map_err(RecvError::Buffer)? == 0 {
                            return Err(RecvError::Closed);
                        } else {
                            continue;
                        }
                    }
                    _ => {
                        self.shared.table.wake_all();
                        return Err(RecvError::Parse(e));
                    }
                },
            };
            if self.filter.check(&msg) {
                return Ok(RecvGuard {
                    shared: &self.shared,
                    buffer,
                });
            } else {
                self.shared.table.wake(&msg);
                buffer = self.handle.wait(buffer).unwrap();
            }
        }
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: BlockingReadBuffer + 'a> {
    shared: &'a SharedData<M, B>,
    buffer: MutexGuard<'a, B>,
}

impl<'a, M: Flat + ?Sized, B: BlockingReadBuffer> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
        match M::from_bytes(&self.buffer) {
            Ok(msg) => self.shared.table.wake(msg),
            Err(_) => self.shared.table.wake_all(),
        }
    }
}

impl<'a, M: Flat + ?Sized, B: BlockingReadBuffer> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(&self.buffer) }
    }
}
