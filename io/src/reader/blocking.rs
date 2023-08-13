use super::{
    endpoint::{Endpoint, EndpointTable, EptHandle, EptId, Filter},
    CommonReader, ReadError, ReadGuard, Reader,
};
use flatty::Flat;
use std::{
    io::Read,
    ops::Deref,
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

impl EptHandle for Condvar {
    fn wake(&self) {
        self.notify_one();
    }
}

pub struct SharedData<M: Flat + ?Sized, R: Read> {
    reader: Mutex<Reader<M, R>>,
    table: EndpointTable<M, Condvar>,
}

pub struct BlockingSharedReader<M: Flat + ?Sized, R: Read> {
    shared: Arc<SharedData<M, R>>,
    filter: Filter<M>,
    id: EptId,
}

impl<M: Flat + ?Sized, R: Read> BlockingSharedReader<M, R> {
    pub fn new(read: R, max_msg_size: usize) -> Self {
        let table = EndpointTable::default();
        let ept = Endpoint {
            filter: Filter::default(),
            handle: Condvar::new(),
        };
        let id = table.insert(ept);
        Self {
            shared: Arc::new(SharedData {
                reader: Mutex::new(Reader::new(read, max_msg_size)),
                table,
            }),
            filter: Filter::default(),
            id,
        }
    }
}

impl<M: Flat + ?Sized, R: Read> Clone for BlockingSharedReader<M, R> {
    fn clone(&self) -> Self {
        let old_ept = self.shared.table.get(self.id).unwrap();
        let filter = old_ept.filter.clone();
        let ept = Endpoint {
            filter: filter.clone(),
            handle: Condvar::new(),
        };
        let id = self.shared.table.insert(ept);
        Self {
            shared: self.shared.clone(),
            filter,
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
            let msg = reader.read_message()?;
            if self.filter.check(&msg) {
                msg.retain();
                break Ok(BlockingSharedReadGuard { owner: reader });
            } else {
                self.shared.table.wake(&msg);
                msg.retain();
                reader = self.shared.table.get(self.id).unwrap().handle.wait(reader).unwrap();
            }
        }
    }
}

pub struct BlockingSharedReadGuard<'a, M: Flat + ?Sized, R: Read> {
    owner: MutexGuard<'a, Reader<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: Read> Drop for BlockingSharedReadGuard<'a, M, R> {
    fn drop(&mut self) {
        let size = self.size();
        self.owner.buffer.skip_occupied(size);
    }
}

impl<'a, M: Flat + ?Sized, R: Read> Deref for BlockingSharedReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.owner.buffer.message().unwrap()
    }
}
