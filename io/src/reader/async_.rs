use super::{
    endpoint::{Endpoint, EndpointTable, EptHandle, EptId, Filter},
    CommonReader, ReadError, ReadGuard, Reader,
};
use flatty::Flat;
use futures::{
    future::FusedFuture,
    io::AsyncRead,
    lock::{Mutex, MutexGuard, MutexLockFuture},
    task::noop_waker,
    FutureExt,
};
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

pub trait AsyncReader<M: Flat + ?Sized>: CommonReader<M> {
    type ReadFuture<'a>: Future<Output = Result<Self::ReadGuard<'a>, ReadError>>
    where
        Self: 'a;

    fn read_message(&mut self) -> Self::ReadFuture<'_>;
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Reader<M, R> {
    fn poll_read_message(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ReadError>> {
        let poll = loop {
            match self.buffer.next_message() {
                Some(result) => break Poll::Ready(result.map(|_| ())),
                None => {
                    if self.buffer.vacant_len() == 0 {
                        assert!(self.buffer.try_extend_vacant());
                    }
                }
            }
            let reader = Pin::new(&mut self.reader);
            match reader.poll_read(cx, self.buffer.vacant_mut()) {
                Poll::Ready(res) => match res {
                    Ok(count) => {
                        if count != 0 {
                            self.buffer.take_vacant(count);
                        } else {
                            break Poll::Ready(Err(ReadError::Eof));
                        }
                    }
                    Err(err) => break Poll::Ready(Err(ReadError::Io(err))),
                },
                Poll::Pending => break Poll::Pending,
            };
        };
        poll
    }

    fn take_message(&mut self) -> ReadGuard<'_, M, R> {
        ReadGuard::new(self)
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> AsyncReader<M> for Reader<M, R> {
    type ReadFuture<'a> = ReadFuture<'a, M, R> where Self: 'a;
    fn read_message(&mut self) -> Self::ReadFuture<'_> {
        ReadFuture { owner: Some(self) }
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for Reader<M, R> {}

pub struct ReadFuture<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    owner: Option<&'a mut Reader<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for ReadFuture<'a, M, R> {}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Future for ReadFuture<'a, M, R> {
    type Output = Result<ReadGuard<'a, M, R>, ReadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self.owner.take().unwrap();
        match owner.poll_read_message(cx) {
            Poll::Ready(res) => Poll::Ready(res.map(|()| owner.take_message())),
            Poll::Pending => {
                self.owner.replace(owner);
                Poll::Pending
            }
        }
    }
}

impl EptHandle for Waker {
    fn wake(&self) {
        Waker::wake_by_ref(self)
    }
}

struct SharedData<M: Flat + ?Sized, R: AsyncRead + Unpin> {
    reader: Mutex<Reader<M, R>>,
    table: EndpointTable<M, Waker>,
}

pub struct AsyncSharedReader<M: Flat + ?Sized, R: AsyncRead + Unpin> {
    shared: Pin<Arc<SharedData<M, R>>>,
    filter: Filter<M>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, R: AsyncRead + Unpin> AsyncSharedReader<M, R> {
    pub fn new(read: R, max_msg_size: usize) -> Self {
        let table = EndpointTable::default();
        let filter = Filter::default();
        let ept = Endpoint {
            filter: filter.clone(),
            handle: noop_waker(),
        };
        let id = table.insert(ept);
        Self {
            shared: Arc::pin(SharedData {
                reader: Mutex::new(Reader::new(read, max_msg_size)),
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

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Clone for AsyncSharedReader<M, R> {
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

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Drop for AsyncSharedReader<M, R> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> CommonReader<M> for AsyncSharedReader<M, R> {
    type ReadGuard<'a> = AsyncSharedReadGuard<'a, M, R> where Self: 'a;
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for AsyncSharedReader<M, R> {}

enum SharedReadState<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    Wait,
    Lock(MutexLockFuture<'a, Reader<M, R>>),
    Read(MutexGuard<'a, Reader<M, R>>),
}

pub struct SharedReadFuture<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    owner: &'a AsyncSharedReader<M, R>,
    state: Option<SharedReadState<'a, M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for SharedReadFuture<'a, M, R> {}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Future for SharedReadFuture<'a, M, R> {
    type Output = Result<AsyncSharedReadGuard<'a, M, R>, ReadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.owner.shared.table.get(self.owner.id).unwrap().handle = cx.waker().clone();
        let mut state = self.state.take().unwrap();
        let mut poll = true;
        while poll {
            (poll, state) = match state {
                SharedReadState::Wait => (true, SharedReadState::Lock(self.owner.shared.reader.lock())),
                SharedReadState::Lock(mut lock) => match lock.poll_unpin(cx) {
                    Poll::Pending => (false, SharedReadState::Lock(lock)),
                    Poll::Ready(reader) => (true, SharedReadState::Read(reader)),
                },
                SharedReadState::Read(mut reader) => match reader.poll_read_message(cx) {
                    Poll::Pending => (false, SharedReadState::Read(reader)),
                    Poll::Ready(Err(e)) => {
                        self.owner.shared.table.wake_all();
                        return Poll::Ready(Err(e));
                    }
                    Poll::Ready(Ok(())) => {
                        let msg = reader.take_message();
                        if self.owner.filter.check(&msg) {
                            msg.retain();
                            return Poll::Ready(Ok(AsyncSharedReadGuard {
                                shared: &self.owner.shared,
                                reader,
                            }));
                        } else {
                            self.owner.shared.table.wake(&msg);
                            msg.retain();
                            (false, SharedReadState::Wait)
                        }
                    }
                },
            };
        }
        assert!(self.state.replace(state).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> FusedFuture for SharedReadFuture<'a, M, R> {
    fn is_terminated(&self) -> bool {
        self.state.is_some()
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> AsyncReader<M> for AsyncSharedReader<M, R> {
    type ReadFuture<'a> = SharedReadFuture<'a, M, R> where Self: 'a;

    fn read_message(&mut self) -> Self::ReadFuture<'_> {
        SharedReadFuture {
            state: Some(SharedReadState::Wait),
            owner: self,
        }
    }
}

pub struct AsyncSharedReadGuard<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    shared: &'a SharedData<M, R>,
    reader: MutexGuard<'a, Reader<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Drop for AsyncSharedReadGuard<'a, M, R> {
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

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Deref for AsyncSharedReadGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.reader.buffer.message().unwrap()
    }
}
