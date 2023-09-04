#[cfg(feature = "io")]
use super::super::IoBuffer;
use super::{
    super::{AsyncReadBuffer, RecvError},
    Endpoint, EndpointTable, EptHandle, EptId, Filter,
};
use flatty::{error::ErrorKind, Flat};
#[cfg(feature = "io")]
use futures::io::AsyncRead;
use futures::{
    lock::{Mutex, MutexGuard},
    pending,
    task::noop_waker,
};
use std::{
    future::poll_fn,
    ops::Deref,
    sync::Arc,
    task::{Poll, Waker},
};

impl EptHandle for Waker {
    fn wake(&self) {
        Waker::wake_by_ref(self)
    }
}

struct SharedData<M: Flat + ?Sized, B: AsyncReadBuffer> {
    buffer: Mutex<B>,
    table: EndpointTable<M, Waker>,
}

pub struct SharedReceiver<M: Flat + ?Sized, B: AsyncReadBuffer> {
    shared: Arc<SharedData<M, B>>,
    filter: Filter<M>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, B: AsyncReadBuffer> SharedReceiver<M, B> {
    pub fn new(buffer: B, max_msg_size: usize) -> Self {
        let table = EndpointTable::default();
        let filter = Filter::default();
        let ept = Endpoint {
            filter: filter.clone(),
            handle: noop_waker(),
        };
        let id = table.insert(ept);
        Self {
            shared: Arc::new(SharedData {
                buffer: Mutex::new(buffer),
                table,
            }),
            filter,
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

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Clone for SharedReceiver<M, B> {
    fn clone(&self) -> Self {
        let filter = self.shared.table.get(self.id).unwrap().filter.clone();
        let ept = Endpoint {
            filter: filter.clone(),
            handle: noop_waker(),
        };
        let id = self.shared.table.insert(ept);
        Self {
            shared: self.shared.clone(),
            filter,
            id,
        }
    }
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Drop for SharedReceiver<M, B> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> Unpin for SharedReceiver<M, B> {}

#[cfg(feature = "io")]
impl<M: Flat + ?Sized + 'static, P: AsyncRead + Unpin> SharedReceiver<M, IoBuffer<P>> {
    pub fn io(pipe: P, max_msg_len: usize) -> Self {
        Self::new(IoBuffer::new(pipe, 2 * max_msg_len.max(M::MIN_SIZE), M::ALIGN), max_msg_len)
    }
}

impl<M: Flat + ?Sized, B: AsyncReadBuffer> SharedReceiver<M, B> {
    pub async fn recv(&mut self) -> Result<RecvGuard<'_, M, B>, RecvError<B::Error>> {
        loop {
            // Register waker.
            poll_fn(|cx| {
                self.shared.table.get(self.id).unwrap().handle = cx.waker().clone();
                Poll::Ready(())
            })
            .await;

            let mut buffer = self.shared.buffer.lock().await;
            let msg = loop {
                match M::from_bytes(&buffer) {
                    Ok(msg) => break msg,
                    Err(e) => match e.kind {
                        ErrorKind::InsufficientSize => {
                            if buffer.read().await.map_err(RecvError::Buffer)? == 0 {
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
                }
            };
            if self.filter.check(&msg) {
                return Ok(RecvGuard {
                    shared: &self.shared,
                    buffer,
                });
            } else {
                self.shared.table.wake(&msg);
                drop(buffer);
                pending!();
            }
        }
    }
}

pub struct RecvGuard<'a, M: Flat + ?Sized, B: AsyncReadBuffer + 'a> {
    shared: &'a SharedData<M, B>,
    buffer: MutexGuard<'a, B>,
}

impl<'a, M: Flat + ?Sized, B: AsyncReadBuffer> Drop for RecvGuard<'a, M, B> {
    fn drop(&mut self) {
        let size = self.size();
        self.buffer.skip(size);
        match M::from_bytes(&self.buffer) {
            Ok(msg) => self.shared.table.wake(msg),
            Err(_) => self.shared.table.wake_all(),
        }
    }
}

impl<'a, M: Flat + ?Sized, B: AsyncReadBuffer> Deref for RecvGuard<'a, M, B> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(&self.buffer) }
    }
}
