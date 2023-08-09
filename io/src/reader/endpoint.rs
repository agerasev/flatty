use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

pub type EptId = usize;

pub trait EptHandle: Sized {
    fn wake(&self);
}

#[derive(Clone)]
pub struct Filter<M: ?Sized> {
    pred: Option<Arc<dyn Fn(&M) -> bool>>,
}
impl<M: ?Sized> Default for Filter<M> {
    fn default() -> Self {
        Self { pred: None }
    }
}
impl<M: ?Sized> Filter<M> {
    pub fn new(pred: Arc<dyn Fn(&M) -> bool>) -> Self {
        Self { pred: Some(pred) }
    }
    pub fn check(&self, value: &M) -> bool {
        match &self.pred {
            Some(fn_) => fn_(value),
            None => true,
        }
    }
}

pub struct Endpoint<M: ?Sized, H: EptHandle> {
    pub filter: Filter<M>,
    pub handle: Option<H>,
}
impl<M: ?Sized, H: EptHandle> Default for Endpoint<M, H> {
    fn default() -> Self {
        Self {
            filter: Filter::default(),
            handle: None,
        }
    }
}

pub struct EndpointTable<M: ?Sized, H: EptHandle> {
    endpoints: Mutex<HashMap<EptId, Endpoint<M, H>>>,
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
    pub fn register(&self, id: EptId, handle: H) {
        self.endpoints.lock().unwrap().get_mut(&id).unwrap().handle = Some(handle);
    }
    pub fn wake(&self, value: &M) {
        let guard = self.endpoints.lock().unwrap();
        for (_, ept) in guard.iter() {
            if ept.filter.check(value) {
                if let Some(h) = &ept.handle {
                    h.wake();
                }
            }
        }
    }
}
