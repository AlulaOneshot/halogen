use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use crate::widget::{DirtyNotify, WidgetId};

pub struct Signal<T: Clone + Send + Sync + 'static> {
    inner: Arc<SignalInner<T>>,
}

struct SignalInner<T: Clone + Send + Sync + 'static> {
    value:       RwLock<T>,
    subscribers: RwLock<HashMap<WidgetId, Weak<dyn DirtyNotify>>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(val: T) -> Self {
        Self {
            inner: Arc::new(SignalInner {
                value:       RwLock::new(val),
                subscribers: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub fn get(&self) -> T {
        self.inner.value.read().unwrap().clone()
    }

    pub fn set(&self, val: T) {
        *self.inner.value.write().unwrap() = val;
        self.notify();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.write().unwrap());
        self.notify();
    }

    pub(crate) fn subscribe(&self, id: WidgetId, notify: Weak<dyn DirtyNotify>) {
        self.inner.subscribers.write().unwrap().insert(id, notify);
    }

    fn notify(&self) {
        self.inner.subscribers.write().unwrap().retain(|_id, weak| {
            if let Some(notify) = weak.upgrade() {
                notify.mark_dirty();
                true
            } else {
                false  // widget was unmounted, prune dead subscriber
            }
        });
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}