use super::{
    endpoint::{Endpoint, EndpointTable, EptHandle, EptId, Filter},
    CommonReader, ReadError, ReadGuard, Reader,
};
use flatty::Flat;
use std::{
    io::Read,
    ops::Deref,
    pin::Pin,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};

pub trait BlockingReader<M: Flat + ?Sized>: CommonReader<M> {
    fn read_message(&mut self) -> Result<Self::ReadGuard<'_>, ReadError>;
}

impl<M: Flat + ?Sized, R: Read> BlockingReader<M> for Reader<M, R> {
    fn read_message(&mut self) -> Result<Self::ReadGuard<'_>, ReadError> {
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
                        break Err(ReadError::Eof);
                    }
                }
                Err(err) => break Err(ReadError::Io(err)),
            }
        }
        .map(|()| ReadGuard::new(self))
    }
}

impl EptHandle for Arc<Condvar> {
    fn wake(&self) {
        self.notify_one();
    }
}

pub struct SharedData<M: Flat + ?Sized, R: Read> {
    reader: Mutex<Reader<M, R>>,
    table: EndpointTable<M, Arc<Condvar>>,
}

pub struct BlockingSharedReader<M: Flat + ?Sized, R: Read> {
    shared: Pin<Arc<SharedData<M, R>>>,
    filter: Filter<M>,
    handle: Arc<Condvar>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, R: Read> BlockingSharedReader<M, R> {
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
                reader: Mutex::new(Reader::new(read, max_msg_size)),
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

impl<M: Flat + ?Sized, R: Read> Clone for BlockingSharedReader<M, R> {
    fn clone(&self) -> Self {
        let filter = self.shared.table.get(self.id).unwrap().filter.clone();
        let handle = Arc::new(Condvar::new());
        let ept = Endpoint {
            filter: filter.clone(),
            handle: Arc::new(Condvar::new()),
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

impl<M: Flat + ?Sized, R: Read> Drop for BlockingSharedReader<M, R> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

impl<M: Flat + ?Sized, R: Read> CommonReader<M> for BlockingSharedReader<M, R> {
    type ReadGuard<'a> = BlockingSharedReadGuard<'a, M, R> where Self: 'a;
}

impl<M: Flat + ?Sized, R: Read> BlockingReader<M> for BlockingSharedReader<M, R> {
    fn read_message(&mut self) -> Result<Self::ReadGuard<'_>, ReadError> {
        let mut reader = self.shared.reader.lock().unwrap();
        loop {
            println!("{}: loop", self.id);
            let msg = match reader.read_message() {
                Ok(msg) => msg,
                Err(e) => {
                    self.shared.table.wake_all(self.id);
                    break Err(e);
                }
            };
            println!("{}: read msg", self.id);
            if self.filter.check(&msg) {
                msg.retain();
                println!("{}: filter self", self.id);
                break Ok(BlockingSharedReadGuard {
                    shared: self.shared.as_ref(),
                    reader,
                    id: self.id,
                });
            } else {
                println!("{}: filter other", self.id);
                self.shared.table.wake(&msg);
                msg.retain();
                println!("{}: wait", self.id);
                reader = self.handle.wait(reader).unwrap();
                println!("{}: wake up", self.id);
            }
        }
    }
}

pub struct BlockingSharedReadGuard<'a, M: Flat + ?Sized, R: Read> {
    shared: Pin<&'a SharedData<M, R>>,
    reader: MutexGuard<'a, Reader<M, R>>,
    id: EptId,
}

impl<'a, M: Flat + ?Sized, R: Read> Drop for BlockingSharedReadGuard<'a, M, R> {
    fn drop(&mut self) {
        let size = self.size();
        self.reader.buffer.skip_occupied(size);
        if let Some(res) = self.reader.buffer.next_message() {
            match res {
                Ok(msg) => self.shared.table.wake(msg),
                Err(_) => self.shared.table.wake_all(self.id),
            }
        }
    }
}

impl<'a, M: Flat + ?Sized, R: Read> Deref for BlockingSharedReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.reader.buffer.message().unwrap()
    }
}
