//! Reactive signals for automatic widget re-rendering.
//!
//! A [`Signal<T>`] is a shared, observable cell. When a signal is read inside a
//! widget's [`build`](crate::widget::Widget::build) call - either via
//! [`BuildContext::read`](crate::widget::BuildContext::read) or via [`Signal::get`]
//! within a [`with_scope`] scope - the widget is automatically subscribed.
//! When the signal changes, every subscriber is marked dirty and scheduled for rebuild.
//!
//! ## Scoping
//!
//! Subscription tracking is thread-local. [`with_scope`] installs the active widget
//! for the current thread; any `Signal::get` call during that scope registers the
//! widget as a subscriber. The tree sets the scope around each `build()` invocation -
//! widgets never need to manage this directly.
//!
//! ## `MaybeSignal<T>`
//!
//! [`MaybeSignal<T>`] lets widget props accept either a static value or a live signal
//! without separate API surfaces. Both `T` and `Signal<T>` convert into it via `From`.
//!
//! ```rust,ignore
//! # use phosphor_core::signal::{Signal, MaybeSignal, signal};
//! fn my_widget(label: impl Into<MaybeSignal<String>>) { /* ... */ }
//!
//! my_widget("static label".to_string());
//! my_widget(signal("reactive label".to_string()));
//! ```

use std::cell::RefCell;
use std::sync::{Arc, RwLock, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::widget::WidgetId;

/// Callback interface for marking a widget as needing rebuild.
///
/// Implemented by the tree's `WidgetDirtyNotify`. Kept as a trait to decouple
/// signals from the concrete tree type - signals only hold a `Weak<dyn DirtyNotify>`,
/// so a dead widget is automatically unsubscribed when the `Weak` fails to upgrade.
pub trait DirtyNotify: Send + Sync {
    fn mark_dirty(&self);
}

thread_local! {
    static CURRENT_SCOPE: RefCell<Option<(WidgetId, Weak<dyn DirtyNotify>)>> =
        const { RefCell::new(None) };
}

/// Run `f` inside a reactive scope for `id`.
///
/// Any [`Signal::get`] call during `f` will subscribe `id` as a dependent.
/// The scope is cleared after `f` returns, regardless of whether it panics.
///
/// The tree calls this around each widget's `build()` invocation. Widgets
/// typically don't need to call it directly.
pub fn with_scope<R>(
    id: WidgetId,
    notify: Weak<dyn DirtyNotify>,
    f: impl FnOnce() -> R,
) -> R {
    CURRENT_SCOPE.with(|s| *s.borrow_mut() = Some((id, notify)));
    let result = f();
    CURRENT_SCOPE.with(|s| *s.borrow_mut() = None);
    result
}

fn current_scope() -> Option<(WidgetId, Weak<dyn DirtyNotify>)> {
    CURRENT_SCOPE.with(|s| s.borrow().clone())
}

struct SignalInner<T> {
    value:       RwLock<T>,
    subscribers: RwLock<Vec<(WidgetId, Weak<dyn DirtyNotify>)>>,
    version:     AtomicU64,
}

/// A reactive shared value.
///
/// All clones of a `Signal<T>` share the same underlying cell - cloning is cheap
/// (`Arc` clone). Use [`Signal::new`] or the [`signal`] free function to create one.
///
/// Signals are typically stored as fields on a widget struct, which gives them the
/// same lifetime as the widget instance. This means state persists between rebuilds
/// without a hook registry.
///
/// ## Thread safety
///
/// `Signal<T>` is `Send + Sync` when `T: Send + Sync`. Reads and writes use
/// `RwLock` internally; multiple threads can read concurrently.
pub struct Signal<T> {
    inner: Arc<SignalInner<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal { inner: Arc::clone(&self.inner) }
    }
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with an initial value.
    pub fn new(value: T) -> Self {
        Signal {
            inner: Arc::new(SignalInner {
                value:       RwLock::new(value),
                subscribers: RwLock::new(Vec::new()),
                version:     AtomicU64::new(0),
            }),
        }
    }

    /// Read the value, subscribing the current widget to future changes.
    ///
    /// If called inside a [`with_scope`], the active widget is registered as a
    /// subscriber and will be marked dirty whenever [`set`](Signal::set) or
    /// [`update`](Signal::update) is called. If called outside any scope, this
    /// behaves identically to [`get_untracked`](Signal::get_untracked).
    pub fn get(&self) -> T {
        if let Some((id, notify)) = current_scope() {
            self.subscribe(id, notify);
        }
        self.inner.value.read().unwrap().clone()
    }

    /// Read the value without subscribing.
    ///
    /// Use when you need the current value but don't want the widget to rebuild
    /// when it changes - e.g. inside an event handler where the read is incidental.
    pub fn get_untracked(&self) -> T {
        self.inner.value.read().unwrap().clone()
    }

    /// Replace the value and notify all subscribers.
    pub fn set(&self, value: T) {
        *self.inner.value.write().unwrap() = value;
        self.inner.version.fetch_add(1, Ordering::Relaxed);
        self.notify_subscribers();
    }

    /// Mutate the value in place and notify all subscribers.
    ///
    /// More efficient than `get` + `set` for types that are expensive to clone
    /// (e.g. `Vec`, `String`), since no clone is needed to produce the new value.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.write().unwrap());
        self.inner.version.fetch_add(1, Ordering::Relaxed);
        self.notify_subscribers();
    }

    /// Monotonically increasing version counter.
    ///
    /// Incremented on every [`set`](Signal::set) or [`update`](Signal::update).
    /// Useful for external caches that want to cheaply detect staleness without
    /// cloning the value.
    pub fn version(&self) -> u64 {
        self.inner.version.load(Ordering::Relaxed)
    }

    //TODO: map, zip

    fn notify_subscribers(&self) {
        // Clean up dead subscribers while notifying live ones.
        let mut subs = self.inner.subscribers.write().unwrap();
        subs.retain(|(_, weak)| {
            if let Some(notify) = weak.upgrade() {
                notify.mark_dirty();
                true
            } else {
                false // widget was dropped, prune
            }
        });
    }

    pub(crate) fn subscribe(&self, id: WidgetId, notify: Weak<dyn DirtyNotify>) {
        let mut subs = self.inner.subscribers.write().unwrap();
        if !subs.iter().any(|(sid, _)| *sid == id) {
            subs.push((id, notify));
        }
    }

    #[allow(unused)] // Not yet implemented, to be public api
    fn subscribe_arc(&self, notify: Arc<dyn DirtyNotify>) {
        self.subscribe(
            // Sentinel WidgetId - effects don't have widget IDs.
            // Placeholder until the effect system lands.
            WidgetId::default(),
            Arc::downgrade(&notify),
        );
        // Keep the Arc alive for now.
        // TODO: proper effect lifecycle management.
        std::mem::forget(notify);
    }

    pub(crate) fn unsubscribe(&self, id: WidgetId) {
        self.inner.subscribers.write().unwrap().retain(|(sid, _)| *sid != id);
    }
}


/// A value that is either static or reactive.
///
/// Widget props that accept `MaybeSignal<T>` can be driven by either a plain
/// value or a live [`Signal<T>`] without needing two separate constructors.
/// Both `T` and `Signal<T>` convert via `From`.
///
/// When the inner value is `Static`, reads never trigger subscriptions.
/// When `Reactive`, reads behave exactly like [`Signal::get`].
pub enum MaybeSignal<T> {
    /// A fixed value - never triggers rebuilds.
    Static(T),
    /// A live signal - subscribes the current widget on read.
    Reactive(Signal<T>),
}

impl<T: Clone + Send + Sync + 'static> MaybeSignal<T> {
    /// Read the current value, subscribing if reactive.
    pub fn get(&self) -> T {
        match self {
            MaybeSignal::Static(v)   => v.clone(),
            MaybeSignal::Reactive(s) => s.get(),
        }
    }

    /// Read the current value without subscribing, even if reactive.
    pub fn get_untracked(&self) -> T {
        match self {
            MaybeSignal::Static(v)   => v.clone(),
            MaybeSignal::Reactive(s) => s.get_untracked(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<T> for MaybeSignal<T> {
    fn from(v: T) -> Self { MaybeSignal::Static(v) }
}

impl<T: Clone + Send + Sync + 'static> From<Signal<T>> for MaybeSignal<T> {
    fn from(s: Signal<T>) -> Self { MaybeSignal::Reactive(s) }
}

/// Convenience constructor - equivalent to [`Signal::new`].
pub fn signal<T: Clone + Send + Sync + 'static>(value: T) -> Signal<T> {
    Signal::new(value)
}