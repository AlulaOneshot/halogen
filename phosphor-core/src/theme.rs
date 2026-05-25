use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Marker trait for a theme key type.
///
/// Implement via the [`theme_key!`] macro rather than manually.
/// Each key type has an associated `Value` type, and is looked up by its
pub trait ThemeKey: 'static {
    type Value: Clone + Send + Sync + 'static;
}

/// Declare a zero-sized theme key and associate it with a value type.
///
/// # Example
///
/// ```rust,ignore
/// # use phosphor_core::theme_key;
/// # use phosphor_core::color::Color;
/// theme_key!(pub AccentColor: Color);
///
/// // Usage
/// # use phosphor_core::theme::Theme;
/// let theme = Theme::new().set(AccentColor, Color::hsl(210.0, 0.8, 0.5));
/// let accent = theme.get(AccentColor);
/// ```
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

/// A type-safe collection of design tokens.
///
/// Values are stored type-erased (`Arc<dyn Any>`) and retrieved by key type.
/// Cloning a `Theme` is cheap - values are reference-counted.
#[derive(Clone, Default)]
pub struct Theme {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Theme {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a token value, returning `self` for chaining.
    pub fn set<K: ThemeKey>(mut self, _key: K, value: K::Value) -> Self {
        self.values.insert(TypeId::of::<K>(), Arc::new(value));
        self
    }

    /// Remove a token, returning `self` for chaining.
    pub fn remove<K: ThemeKey>(mut self, _key: K) -> Self {
        self.values.remove(&TypeId::of::<K>());
        self
    }

    /// Set a token value by mutable reference.
    pub fn set_mut<K: ThemeKey>(&mut self, _key: K, value: K::Value) {
        self.values.insert(TypeId::of::<K>(), Arc::new(value));
    }

    /// Remove a token by mutable reference.
    pub fn remove_mut<K: ThemeKey>(&mut self, _key: K) {
        self.values.remove(&TypeId::of::<K>());
    }

    /// Look up a token value. Returns `None` if the key has not been set.
    pub fn get<K: ThemeKey>(&self, _key: K) -> Option<&K::Value> {
        self.values
            .get(&TypeId::of::<K>())
            .and_then(|v| v.downcast_ref::<K::Value>())
    }

    /// Look up a token value, returning `fallback` if absent.
    pub fn get_or<K: ThemeKey>(&self, key: K, fallback: K::Value) -> K::Value {
        self.get(key).cloned().unwrap_or(fallback)
    }

    /// Returns `true` if the key has a value set.
    pub fn contains<K: ThemeKey>(&self, _key: K) -> bool {
        self.values.contains_key(&TypeId::of::<K>())
    }

    /// Merge `other` on top of `self`, returning the combined theme.
    ///
    /// Tokens present in `other` overwrite those in `self`. Tokens only in
    /// `self` are preserved. This is the correct way to build theme inheritance.
    pub fn merge(mut self, other: &Theme) -> Self {
        for (k, v) in &other.values {
            self.values.insert(*k, Arc::clone(v));
        }
        self
    }

    /// Mutating variant of [`merge`](Theme::merge).
    pub fn extend(&mut self, other: &Theme) {
        for (k, v) in &other.values {
            self.values.insert(*k, Arc::clone(v));
        }
    }

    /// Iterate over keys whose values differ between `self` and `other`.
    ///
    /// Uses [`Arc::ptr_eq`] for comparison - value equality is not checked,
    /// only identity. Keys present in `other` but not `self` are included.
    /// Keys present in `self` but not `other` are not.
    pub fn diff<'a>(&'a self, other: &'a Theme) -> impl Iterator<Item = TypeId> + 'a {
        other.values.keys()
            .filter(|k| self.values.get(k).map(|v| !Arc::ptr_eq(v, other.values.get(k).unwrap())).unwrap_or(true))
            .copied()
    }

    /// Number of tokens currently set.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterate over all set key `TypeId`s.
    pub fn keys(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.values.keys().copied()
    }
}

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Theme({} tokens)", self.values.len())
    }
}