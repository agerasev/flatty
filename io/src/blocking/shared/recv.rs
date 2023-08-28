use crate::{
    blocking::BlockingReceiver,
    common::{
        shared::{Endpoint, EndpointTable, EptHandle, EptId, Filter},
        CommonReceiver, Receiver, RecvError,
    },
};
use flatty::Flat;
use std::{
    io::Read,
    ops::Deref,
    pin::Pin,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};

impl EptHandle for Arc<Condvar> {
    fn wake(&self) {
        self.notify_one();
    }
}

struct SharedData<M: Flat + ?Sized, R: Read> {
    reader: Mutex<Receiver<M, R>>,
    table: EndpointTable<M, Arc<Condvar>>,
}

pub struct SharedReceiver<M: Flat + ?Sized, R: Read> {
    shared: Pin<Arc<SharedData<M, R>>>,
    filter: Filter<M>,
    handle: Arc<Condvar>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, R: Read> SharedReceiver<M, R> {
    pub fn new(read: R, max_msg_size: usize) -> Self {
        let table = EndpointTable::default();
        let filter = Filter::default();
        let handle = Arc::new(Condvar::new());
        let ept = Endpoint {
            filter: filter.clone(),
            handle: handle.clone(),
        };
        let id = table.insert(ept);
        Self {
            shared: Arc::pin(SharedData {
                reader: Mutex::new(Receiver::new(read, max_msg_size)),
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

impl<M: Flat + ?Sized, R: Read> Clone for SharedReceiver<M, R> {
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

impl<M: Flat + ?Sized, R: Read> Drop for SharedReceiver<M, R> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

impl<M: Flat + ?Sized, R: Read> CommonReceiver<M> for SharedReceiver<M, R> {
    type RecvGuard<'a> = SharedRecvGuard<'a, M, R> where Self: 'a;
}

impl<M: Flat + ?Sized, R: Read> BlockingReceiver<M> for SharedReceiver<M, R> {
    fn recv(&mut self) -> Result<Self::RecvGuard<'_>, RecvError> {
        let mut reader = self.shared.reader.lock().unwrap();
        loop {
            let msg = match reader.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    self.shared.table.wake_all();
                    break Err(e);
                }
            };
            if self.filter.check(&msg) {
                msg.retain();
                break Ok(SharedRecvGuard {
                    shared: &self.shared,
                    reader,
                });
            } else {
                self.shared.table.wake(&msg);
                msg.retain();
                reader = self.handle.wait(reader).unwrap();
            }
        }
    }
}

pub struct SharedRecvGuard<'a, M: Flat + ?Sized, R: Read> {
    shared: &'a SharedData<M, R>,
    reader: MutexGuard<'a, Receiver<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: Read> Drop for SharedRecvGuard<'a, M, R> {
    fn drop(&mut self) {
        let size = self.size();
        self.reader.buffer.skip_occupied(size);
        if let Some(res) = self.reader.buffer.next_message() {
            match res {
                Ok(msg) => self.shared.table.wake(msg),
                Err(_) => self.shared.table.wake_all(),
            }
        }
    }
}

impl<'a, M: Flat + ?Sized, R: Read> Deref for SharedRecvGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.reader.buffer.message().unwrap()
    }
}
