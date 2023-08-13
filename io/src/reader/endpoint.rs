use owning_ref::OwningRefMut;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

pub type EptId = usize;

pub trait EptHandle: Sized {
    fn wake(&self);
}

pub type FilterFn<M> = dyn Fn(&M) -> bool;

pub struct Filter<M: ?Sized> {
    pred: Option<Arc<FilterFn<M>>>,
}
impl<M: ?Sized> Clone for Filter<M> {
    fn clone(&self) -> Self {
        Filter { pred: self.pred.clone() }
    }
}
impl<M: ?Sized> Default for Filter<M> {
    fn default() -> Self {
        Self { pred: None }
    }
}
impl<M: ?Sized> Filter<M> {
    pub fn new(pred: Arc<FilterFn<M>>) -> Self {
        Self { pred: Some(pred) }
    }
    pub fn check(&self, value: &M) -> bool {
        match &self.pred {
            Some(fn_) => fn_(value),
            None => true,
        }
    }
}

#[derive(Default)]
pub struct Endpoint<M: ?Sized, H: EptHandle> {
    pub filter: Filter<M>,
    pub handle: H,
}

type EptMap<M, H> = HashMap<EptId, Endpoint<M, H>>;
pub struct EndpointTable<M: ?Sized, H: EptHandle> {
    endpoints: Mutex<EptMap<M, H>>,
    counter: AtomicUsize,
}
impl<M: ?Sized, H: EptHandle> Default for EndpointTable<M, H> {
    fn default() -> Self {
        Self {
            endpoints: Mutex::new(HashMap::new()),
            counter: AtomicUsize::new(0),
        }
    }
}

impl<M: ?Sized, H: EptHandle> EndpointTable<M, H> {
    pub fn insert(&self, ept: Endpoint<M, H>) -> EptId {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        assert!(self.endpoints.lock().unwrap().insert(id, ept).is_none());
        id
    }
    pub fn remove(&self, id: EptId) {
        assert!(self.endpoints.lock().unwrap().remove(&id).is_some());
    }
    pub fn get(&self, id: EptId) -> Option<OwningRefMut<MutexGuard<'_, EptMap<M, H>>, Endpoint<M, H>>> {
        OwningRefMut::new(self.endpoints.lock().unwrap())
            .try_map_mut(move |o| o.get_mut(&id).ok_or(()))
            .ok()
    }
    pub fn wake(&self, value: &M) {
        let guard = self.endpoints.lock().unwrap();
        for (_, ept) in guard.iter() {
            if ept.filter.check(value) {
                ept.handle.wake();
            }
        }
    }
}
