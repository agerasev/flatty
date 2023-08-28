#![allow(clippy::type_complexity)]

mod endpoint;

use std::sync::Arc;

pub struct Filter<M: ?Sized> {
    pred: Option<Arc<dyn Fn(&M) -> bool + Sync + Send>>,
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
    pub fn new<F: Fn(&M) -> bool + Sync + Send + 'static>(pred: F) -> Self {
        Self {
            pred: Some(Arc::new(pred)),
        }
    }
    pub fn check(&self, value: &M) -> bool {
        match &self.pred {
            Some(fn_) => fn_(value),
            None => true,
        }
    }
}

pub use endpoint::*;
