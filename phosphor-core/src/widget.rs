//! Core widget trait and associated contexts.
//!
//! Every UI element in Phosphor implements [`Widget`]. A widget is a pure
//! description: it holds state (as [`Signal<T>`] fields) and produces a subtree
//! via [`build`](Widget::build). The tree owns all widget instances and drives
//! rebuild, layout, paint, and event dispatch.
//!
//! ## Lifecycle
//!
//! 1. **Mount** - widget is inserted into the tree; `build()` is called and
//!    children are mounted recursively. The widget is subscribed to any signals
//!    read during `build()`.
//! 2. **Rebuild** - when a subscribed signal fires, the widget is marked dirty.
//!    On the next `rebuild()` pass, `build()` is called again and the result is
//!    reconciled against the existing children.
//! 3. **Layout** - after rebuilds settle, `layout_style()` is passed to Taffy
//!    and absolute bounds are resolved.
//! 4. **Paint** - `paint()` is called depth-first with resolved bounds.
//! 5. **Event** - events are dispatched depth-first; each widget returns
//!    [`EventResult::Consumed`] or [`EventResult::Bubble`].
//! 6. **Unmount** - when reconciliation removes a widget, it is dropped.
//!
//! ## Storing state
//!
//! State lives as [`Signal<T>`] fields on the widget struct. Because the tree
//! keeps the widget instance alive between rebuilds, signal values persist
//! without a separate hook registry.
//!
//! ```rust,ignore
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Widget for Counter {
//!     fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode> {
//!         let n = cx.read(&self.count); // subscribes; rebuilds when count changes
//!         vec![text(n.to_string()).into()]
//!     }
//! }
//! ```

use crate::event::{Event, EventResult};
use crate::handler::{Handler, HandlerExecutor};
use crate::signal::{DirtyNotify, Signal};
use crate::style::{BorderStyle, BoxShadow, Corners, MeasureFunc, Rect, Style, Texture, ViewportSize, WidgetStyle};
use crate::theme::Theme;
use downcast_rs::{impl_downcast, Downcast};
use slotmap::new_key_type;
use std::sync::Weak;

new_key_type! {
    /// Stable, generation-checked handle for a widget instance in the tree.
    /// Backed by `slotmap::SlotMap` - accessing a slot after the widget is
    /// removed returns `None` rather than aliasing a new widget at the same index.
    pub struct WidgetId;
}

/// The core trait for all UI elements.
///
/// Implementors should be `Send + Sync + 'static`. All methods have default
/// implementations so you only override what you need.
pub trait Widget: Downcast + Send + Sync + 'static {
    /// Return this widget's child subtree.
    ///
    /// Called once on mount and again whenever a signal read inside a previous
    /// `build()` fires. The tree diffs the returned children against the existing
    /// ones and only mounts, updates, or removes what changed.
    ///
    /// Reads to [`Signal<T>`] via [`BuildContext::read`] (or [`Signal::get`] within
    /// the active scope) are automatically tracked - the widget will be scheduled for
    /// rebuild whenever those signals change.
    ///
    /// Return an empty `Vec` for leaf widgets.
    fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode>;

    /// Return this widget's style.
    ///
    /// The default implementation returns a lazily-initialized `WidgetStyle::new()`.
    /// Override to return a custom style or a reference to one stored on the struct.
    ///
    /// The returned reference must be `'static` or tied to `&self`. The
    /// `OnceLock<WidgetStyle>` pattern in the default handles the `'static` case
    /// without requiring a `const` default (which `Vec` prevents).
    fn style(&self) -> &dyn Style {
        static DEFAULT: std::sync::OnceLock<WidgetStyle> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(WidgetStyle::new)
    }

    /// Return the Taffy layout style for this widget.
    ///
    /// The default calls `self.style().to_taffy(viewport)`. Override only if you
    /// need viewport-dependent layout logic that isn't expressible through [`Style`].
    fn layout_style(&self, viewport: ViewportSize) -> taffy::Style {
        self.style().to_taffy(viewport)
    }

    /// Paint this widget using resolved bounds from the layout pass.
    ///
    /// Called after layout, before children paint. The default calls
    /// [`PaintContext::paint_style`] with `self.style()`. If you override this,
    /// you're responsible for calling `paint_style` yourself if you want the
    /// standard background/border/shadow rendering.
    fn paint(&self, cx: &mut PaintContext) {
        cx.paint_style(self.style());
    }

    /// Handle an input event.
    ///
    /// Called after all children have had a chance to consume the event
    /// (deepest widget wins). Return [`EventResult::Consumed`] to stop propagation
    /// or [`EventResult::Bubble`] to pass it up. The default always bubbles.
    fn on_event(&self, event: &Event, cx: &mut EventContext) -> EventResult {
        let _ = event;
        let _ = cx;
        EventResult::Bubble
    }

    /// A human-readable name used in debug output and tests.
    /// Defaults to the fully-qualified type name via `std::any::type_name`.
    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Provide a custom text-measurement function for layout, or for anything that may need runtime layout calculations.
    /// Return `Some` for widgets that contain text and need Taffy to call back
    /// for size measurements. For all other widgets, return `None` (the default).
    fn measure(&self) -> Option<MeasureFunc> {
        None
    }

    //TODO: Accessibility
}

impl_downcast!(Widget);

/// A widget instance paired with an optional stable identity key.
///
/// Returned from [`Widget::build`]. The key is used during reconciliation:
/// keyed nodes are matched by key rather than position, which preserves
/// widget identity across reordering (useful for dynamic lists).
pub struct WidgetNode {
    pub widget: Box<dyn Widget>,
    /// Optional key for stable identity across reconciliation.
    pub key: Option<String>,
}

impl WidgetNode {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            key: None,
        }
    }

    /// Assign a key for stable identity across reconciliation.
    ///
    /// Use for items in dynamic lists where order may change.
    /// The key only needs to be unique among siblings.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl<W: Widget + 'static> From<W> for WidgetNode {
    fn from(w: W) -> Self {
        WidgetNode::new(w)
    }
}

/// Context passed to [`Widget::build`].
///
/// Provides signal reading (with automatic subscription tracking), theme access,
/// viewport size, and this widget's own [`WidgetId`].
pub struct BuildContext<'a> {
    pub(crate) widget_id: WidgetId,
    pub(crate) theme: &'a Theme,
    /// Used to notify if the widget is dirty
    pub(crate) dirty_notify: Weak<dyn DirtyNotify>,
    pub(crate) viewport: ViewportSize,
}

impl<'a> BuildContext<'a> {
    /// Read a signal value and subscribe this widget to future changes.
    ///
    /// Equivalent to calling `signal.get()` inside a [`with_scope`](crate::signal::with_scope)
    /// scope. The widget will be scheduled for rebuild whenever the signal is updated.
    pub fn read<T: Clone + Send + Sync + 'static>(&self, signal: &Signal<T>) -> T {
        signal.subscribe(self.widget_id, self.dirty_notify.clone());
        signal.get()
    }

    /// The current theme.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Logical size of the window/surface.
    pub fn viewport(&self) -> ViewportSize {
        self.viewport
    }

    /// This widget's stable ID in the tree.
    pub fn widget_id(&self) -> WidgetId {
        self.widget_id
    }
}

/// Context passed to [`Widget::paint`].
///
/// Contains the widget's resolved absolute screen bounds and access to the
/// [`PaintSink`] for issuing draw commands.
pub struct PaintContext<'a> {
    pub bounds: Rect,
    pub theme: &'a Theme,
    #[allow(unused)] //TODO: renderer not yet implemented
    pub(crate) renderer:  &'a mut dyn PaintSink,
}

/// Absolute screen coordinates for this widget, resolved after layout.
impl<'a> PaintContext<'a> {
    /// Emit draw commands for a widget's background, border, shadows, and opacity.
    ///
    /// **Currently a stub.** The `PaintSink` API is not yet implemented.
    pub fn paint_style(&mut self, style: &dyn Style) {
        let paint = style.paint();
        let radii = paint.border.radius;

        if paint.opacity == 0.0 {
            return;
        }

        let use_layer = paint.opacity < 1.0;
        if use_layer {
            self.renderer.push_layer(paint.opacity);
        }

        // 1. Shadows
        for shadow in paint.shadows.iter().filter(|s| !s.inset) {
            self.renderer.draw_shadow(self.bounds, radii, shadow);
        }

        // 2. Background
        self.renderer.fill_rrect(self.bounds, radii, &paint.background.0);

        // 3. Border
        //self.renderer.stroke_rrect(self.bounds, radii, &paint.border);

        // 4. Inset shadows
        for shadow in paint.shadows.iter().filter(|s| s.inset) {
            self.renderer.draw_shadow(self.bounds, radii, shadow);
        }

        if use_layer {
            self.renderer.pop_layer();
        }
    }

    //TODO: draw commands (text, image, custom path, etc.)
}

/// Implemented by renderers in `phosphor-render`.
///
/// Widgets issue draw commands through [`PaintContext`], which delegates to
/// the active `PaintSink`. The `SkiaRenderer` and `HeadlessRenderer` both
/// implement this trait.
///
/// ## Expected Paint Order:
///
/// 1. Drop Shadows (non-inset), as they are behind the element
/// 2. Background fill
/// 3. Border
/// 4. Inset Shadows, above border, inside bounds.
///
/// ## Clipping
///
/// Clips are managed in a stack via push_clip_rrect and pop_clip_rrect.
///
/// ## Opacity
///
/// Opacity is handles by rendering into an offscreen layer at full opacity (if the set opacity isn't 0.0 or 1.0), then copying it into the main buffer at full opacity.
pub trait PaintSink {
    /// Draw a box shadow behind or inside a rounded rect.
    ///
    /// For drop shadows (`BoxShadow::inset == false`), this must be called
    /// **before** [`fill_rrect`](PaintSink::fill_rrect) so the fill paints over
    /// the shadow where they overlap.
    ///
    /// For inset shadows (`BoxShadow::inset == true`), this must be called
    /// **after** [`stroke_rrect`](PaintSink::stroke_rrect) so the shadow renders
    /// inside the widget's bounds on top of the border.
    ///
    /// The `rect` and `radii` describe the widget's own bounds — the renderer is
    /// responsible for expanding by `shadow.spread` and offsetting by `shadow.offset`.
    fn draw_shadow(&mut self, rect: Rect, radii: Corners<f32>, shadow: &BoxShadow);

    /// Fill a rounded rect with a [`Texture`],
    ///
    /// Handles all texture variants: solid color, linear/radial/conic/mesh gradients.
    /// Gradient coordinates are resolved relative to `rect`.
    ///
    /// Used for widget backgrounds. Called after drop shadows, before borders.
    fn fill_rrect(&mut self, rect: Rect, radii: Corners<f32>, texture: &Texture);

    /// Stroke the border of a rounded rect.
    ///
    /// [`BorderStyle`] carries per-edge widths and textures, allowing each edge
    /// to have an independent color or gradient. Corner pixels are split between
    /// their two adjacent edges at a 45° diagonal.
    ///
    /// Called after [`fill_rrect`](PaintSink::fill_rrect).
    fn stroke_rrect(&mut self, rect: Rect, radii: Corners<f32>, border: &BorderStyle);

    /// Push a rounded rect clip onto the clip stack.
    ///
    /// All subsequent draw calls are clipped to the intersection of this rect and
    /// any previously active clips. Must be paired with a matching
    /// [`pop_clip`](PaintSink::pop_clip).
    ///
    /// Used by the tree for widgets with [`Overflow::Hidden`]. The clip is applied
    /// around the widget's **children**, not the widget itself — the widget's own
    /// background and border are not clipped.
    fn push_clip_rrect(&mut self, rect: Rect, radii: Corners<f32>);

    /// Pop the most recently pushed clip from the clip stack, restoring the
    /// previous clip region.
    ///
    /// Must be called once for every [`push_clip_rrect`](PaintSink::push_clip_rrect).
    fn pop_clip(&mut self);

    /// Begin rendering into an isolated offscreen layer at the given opacity.
    ///
    /// All draw calls until the matching [`pop_layer`](PaintSink::pop_layer) are
    /// rendered into this layer. On pop, the layer is composited onto the parent
    /// surface multiplied by `opacity`.
    ///
    /// `opacity` is clamped to `[0.0, 1.0]`. Layers may be nested.
    ///
    /// Callers should skip `push_layer`/`pop_layer` entirely when `opacity == 1.0`
    /// to avoid the offscreen surface allocation.
    fn push_layer(&mut self, opacity: f32);

    /// Composite the current offscreen layer onto the parent surface and discard it.
    ///
    /// Must be called once for every [`push_layer`](PaintSink::push_layer).
    fn pop_layer(&mut self);
}

/// Context passed to [`Widget::on_event`].
///
/// Provides bounds, theme, and the ability to fire a [`Handler`] without
/// coupling to a specific async runtime.
pub struct EventContext<'a> {
    /// This widget's stable ID in the tree.
    pub widget_id: WidgetId,
    /// Absolute screen coordinates, available for hit-testing.
    /// While a click event will not trigger if it does not hit the defined bounds of the shape, this can be used for extra logic or unusual shapes.
    pub bounds: Rect,
    pub theme: &'a Theme,
    /// Runtime-agnostic handler executor. See [`HandlerExecutor`].
    pub executor: &'a dyn HandlerExecutor,
}

impl<'a> EventContext<'a> {
    /// Execute a handler through the active [`HandlerExecutor`].
    ///
    /// For sync handlers this runs immediately. For async handlers the executor
    /// is responsible for spawning the future. With [`SyncOnlyExecutor`](crate::handler::SyncOnlyExecutor),
    /// calling this with an async handler will panic.
    pub fn fire(&self, handler: &Handler) {
        self.executor.fire(handler);
    }

    pub fn theme(&self) -> &Theme {
        self.theme
    }
}