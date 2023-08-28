use super::{
    super::{AsyncReceiver, CommonReceiver, Receiver, RecvError},
    Endpoint, EndpointTable, EptHandle, EptId, Filter,
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

impl EptHandle for Waker {
    fn wake(&self) {
        Waker::wake_by_ref(self)
    }
}

struct SharedData<M: Flat + ?Sized, R: AsyncRead + Unpin> {
    reader: Mutex<Receiver<M, R>>,
    table: EndpointTable<M, Waker>,
}

pub struct SharedReceiver<M: Flat + ?Sized, R: AsyncRead + Unpin> {
    shared: Pin<Arc<SharedData<M, R>>>,
    filter: Filter<M>,
    id: EptId,
}

impl<M: Flat + ?Sized + 'static, R: AsyncRead + Unpin> SharedReceiver<M, R> {
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
                reader: Mutex::new(Receiver::new(read, max_msg_size)),
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

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Clone for SharedReceiver<M, R> {
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

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Drop for SharedReceiver<M, R> {
    fn drop(&mut self) {
        self.shared.table.remove(self.id);
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> CommonReceiver<M> for SharedReceiver<M, R> {
    type RecvGuard<'a> = SharedRecvGuard<'a, M, R> where Self: 'a;
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for SharedReceiver<M, R> {}

enum SharedRecvState<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    Wait,
    Lock(MutexLockFuture<'a, Receiver<M, R>>),
    Read(MutexGuard<'a, Receiver<M, R>>),
}

pub struct SharedRecvFuture<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    owner: &'a SharedReceiver<M, R>,
    state: Option<SharedRecvState<'a, M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Unpin for SharedRecvFuture<'a, M, R> {}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Future for SharedRecvFuture<'a, M, R> {
    type Output = Result<SharedRecvGuard<'a, M, R>, RecvError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.owner.shared.table.get(self.owner.id).unwrap().handle = cx.waker().clone();
        let mut state = self.state.take().unwrap();
        let mut poll = true;
        while poll {
            (poll, state) = match state {
                SharedRecvState::Wait => (true, SharedRecvState::Lock(self.owner.shared.reader.lock())),
                SharedRecvState::Lock(mut lock) => match lock.poll_unpin(cx) {
                    Poll::Pending => (false, SharedRecvState::Lock(lock)),
                    Poll::Ready(reader) => (true, SharedRecvState::Read(reader)),
                },
                SharedRecvState::Read(mut reader) => match reader.poll_receive(cx) {
                    Poll::Pending => (false, SharedRecvState::Read(reader)),
                    Poll::Ready(Err(e)) => {
                        self.owner.shared.table.wake_all();
                        return Poll::Ready(Err(e));
                    }
                    Poll::Ready(Ok(())) => {
                        let msg = reader.take_message();
                        if self.owner.filter.check(&msg) {
                            msg.retain();
                            return Poll::Ready(Ok(SharedRecvGuard {
                                shared: &self.owner.shared,
                                reader,
                            }));
                        } else {
                            self.owner.shared.table.wake(&msg);
                            msg.retain();
                            (false, SharedRecvState::Wait)
                        }
                    }
                },
            };
        }
        assert!(self.state.replace(state).is_none());
        Poll::Pending
    }
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> FusedFuture for SharedRecvFuture<'a, M, R> {
    fn is_terminated(&self) -> bool {
        self.state.is_some()
    }
}

impl<M: Flat + ?Sized, R: AsyncRead + Unpin> AsyncReceiver<M> for SharedReceiver<M, R> {
    type RecvFuture<'a> = SharedRecvFuture<'a, M, R> where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        SharedRecvFuture {
            state: Some(SharedRecvState::Wait),
            owner: self,
        }
    }
}

pub struct SharedRecvGuard<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> {
    shared: &'a SharedData<M, R>,
    reader: MutexGuard<'a, Receiver<M, R>>,
}

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Drop for SharedRecvGuard<'a, M, R> {
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

impl<'a, M: Flat + ?Sized, R: AsyncRead + Unpin> Deref for SharedRecvGuard<'a, M, R> {
    type Target = M;
    fn deref(&self) -> &M {
        self.reader.buffer.message().unwrap()
    }
}
