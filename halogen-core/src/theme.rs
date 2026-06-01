use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub trait ThemeKey: 'static {
    type Value: Clone + Send + Sync + 'static;
}

#[macro_export]
macro_rules! theme_key {
    ($vis:vis $name:ident : $ty:ty) => {
        #[derive(Debug, Clone, Copy)]
        $vis struct $name;

        impl $crate::theme::ThemeKey for $name {
            type Value = $ty;
        }
    };
}

#[derive(Clone, Default)]
pub struct Theme {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Theme {
    pub fn new() -> Self { Self::default() }

    pub fn set<K: ThemeKey>(mut self, _key: K, value: K::Value) -> Self {
        self.values.insert(TypeId::of::<K>(), Arc::new(value));
        self
    }

    pub fn set_mut<K: ThemeKey>(&mut self, _key: K, value: K::Value) {
        self.values.insert(TypeId::of::<K>(), Arc::new(value));
    }

    pub fn get<K: ThemeKey>(&self, _key: K) -> Option<&K::Value> {
        self.values
            .get(&TypeId::of::<K>())
            .and_then(|v| v.downcast_ref::<K::Value>())
    }

    pub fn get_or<K: ThemeKey>(&self, key: K, fallback: K::Value) -> K::Value {
        self.get(key).cloned().unwrap_or(fallback)
    }

    pub fn contains<K: ThemeKey>(&self, _key: K) -> bool {
        self.values.contains_key(&TypeId::of::<K>())
    }

    pub fn merge(mut self, other: &Theme) -> Self {
        for (k, v) in &other.values {
            self.values.insert(*k, Arc::clone(v));
        }
        self
    }
}