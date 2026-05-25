//! Sync and async event callbacks with runtime-agnostic dispatch.
//!
//! [`Handler`] stores a callable - either a plain `Fn()` or an async closure -
//! as a type-erased `Arc`. Widgets store `Handler` values in their structs and
//! fire them via [`EventContext::fire`](crate::widget::EventContext::fire).
//!
//! Execution is delegated to a [`HandlerExecutor`], which decouples the core
//! crate from any specific async runtime. Platform code provides a
//! tokio-backed executor; tests use [`SyncOnlyExecutor`].

use std::sync::Arc;
use futures_core::future::BoxFuture;

/// A type-erased callback, either synchronous or async.
///
/// Both variants are `Clone` (backed by `Arc`) and `Send + Sync`.
/// Use [`Handler::sync`], [`Handler::async_fn`], or the [`async_handler`]
/// convenience function to construct one.
#[derive(Clone)]
pub enum Handler {
    Sync(Arc<dyn Fn() + Send + Sync + 'static>),
    Async(Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static>),
}

impl Handler {
    /// Construct a synchronous handler from any `Fn() + Send + Sync`.
    pub fn sync(f: impl Fn() + Send + Sync + 'static) -> Self {
        Handler::Sync(Arc::new(f))
    }

    /// Construct an async handler from an async closure.
    ///
    /// The future is boxed and pinned automatically. The closure must be
    /// `Send + Sync + 'static`, and the returned future must be `Send + 'static`.
    pub fn async_fn<F, Fut>(f: F) -> Self
    where
        F:   Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Handler::Async(Arc::new(move || Box::pin(f())))
    }

    /// Returns `true` if this is an [`Handler::Async`] variant.
    pub fn is_async(&self) -> bool {
        matches!(self, Handler::Async(_))
    }
}

impl std::fmt::Debug for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Handler::Sync(_)  => write!(f, "Handler::Sync(..)"),
            Handler::Async(_) => write!(f, "Handler::Async(..)"),
        }
    }
}

/// Any `Fn() + Send + Sync` converts into a [`Handler::Sync`].
impl<F: Fn() + Send + Sync + 'static> From<F> for Handler {
    fn from(f: F) -> Self { Handler::sync(f) }
}

/// Convenience constructor for handlers. Equivalent to [`Handler::sync`].
///
/// ```rust,ignore
/// let h = handler(|| {
///     do_thing();
/// });
/// ```
pub fn handler<F: Fn() + Send + Sync + 'static>(f: F) -> Handler
{
    Handler::sync(f)
}

/// Convenience constructor for async handlers. Equivalent to [`Handler::async_fn`].
///
/// ```rust,ignore
/// let h = async_handler(|| async {
///     do_something_async().await;
/// });
/// ```
pub fn async_handler<F, Fut>(f: F) -> Handler
where
    F:   Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Handler::async_fn(f)
}

/// Drives handler execution, decoupled from any specific async runtime.
///
/// Platform code provides a tokio-backed implementation. Headless tests use
/// [`SyncOnlyExecutor`]. Widgets never interact with the executor directly -
/// they call [`EventContext::fire`](crate::widget::EventContext::fire), which
/// delegates to whatever executor the tree was given.
pub trait HandlerExecutor: Send + Sync {
    fn fire(&self, handler: &Handler);
}

/// An executor that only supports synchronous handlers.
///
/// Safe to use in tests and in any context where tokio is not available.
///
/// # Panics
///
/// Panics if asked to fire an [`Handler::Async`] handler.
pub struct SyncOnlyExecutor;

impl HandlerExecutor for SyncOnlyExecutor {
    fn fire(&self, handler: &Handler) {
        match handler {
            Handler::Sync(f)  => f(),
            Handler::Async(_) => panic!(
                "SyncOnlyExecutor cannot drive async handlers. \
                 Use a tokio-backed executor in platform code."
            ),
        }
    }
}