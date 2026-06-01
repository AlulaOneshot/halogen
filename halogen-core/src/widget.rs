use crate::style::{Style, ViewportSize};
use slotmap::new_key_type;
use std::sync::Weak;
use crate::event::{Event, EventResult};
use crate::geometry::MeasureFunc;
use crate::signal::Signal;
use crate::theme::Theme;
use crate::tree::Scheduler;
use std::future::Future;
use crate::paint::{PaintCommand, PaintContext};

new_key_type! { pub struct WidgetId; }

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum WidgetKey {
    Int(u64),
    Str(&'static str),
}

impl From<u64> for WidgetKey {
    fn from(n: u64) -> Self {
        WidgetKey::Int(n)
    }
}

impl From<&'static str> for WidgetKey {
    fn from(s: &'static str) -> Self {
        WidgetKey::Str(s)
    }
}

/// Hold as weak by signals, use to mark subscribers dirty.
pub trait DirtyNotify: Send + Sync {
    fn mark_dirty(&self);
}

pub struct BuildContext<'a> {
    pub(crate) id: WidgetId,
    pub(crate) theme: &'a Theme,
    pub(crate) viewport: ViewportSize,
    pub(crate) dirty_notify: Weak<dyn DirtyNotify>,
}

impl<'a> BuildContext<'a> {
    pub fn id(&self) -> WidgetId {
        self.id
    }
    pub fn theme(&self) -> &Theme {
        self.theme
    }
    pub fn viewport(&self) -> ViewportSize {
        self.viewport
    }

    pub fn read<T: Clone + Send + Sync + 'static>(&self, signal: &Signal<T>) -> T {
        signal.subscribe(self.id, self.dirty_notify.clone());
        signal.get()
    }
}

pub struct LifecycleContext<'a> {
    pub id: WidgetId,
    pub theme: &'a Theme,
    pub viewport: ViewportSize,
    pub(crate) scheduler: &'a dyn Scheduler,
}

impl<'a> LifecycleContext<'a> {
    pub fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        self.scheduler.spawn(Box::new(fut));
    }

    pub fn mark_dirty(&self) {
        self.scheduler.mark_dirty(self.id);
    }
}

pub struct EventContext<'a> {
    pub id: WidgetId,
    pub bounds: crate::geometry::Rect,
    pub theme: &'a Theme,
    pub(crate) scheduler: &'a dyn Scheduler,
}

impl<'a> EventContext<'a> {
    pub fn mark_dirty(&self) {
        self.scheduler.mark_dirty(self.id);
    }
}

pub trait PaintSink {
    fn push(&mut self, command: PaintCommand);
}

pub trait Widget: downcast_rs::Downcast + Send + Sync + CloneWidget {
    fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode>;

    fn on_mount(&mut self, _cx: &mut LifecycleContext) {}
    fn on_unmount(&mut self, _cx: &mut LifecycleContext) {}
    fn on_event(&self, _event: &Event, _cx: &mut EventContext) -> EventResult {
        EventResult::Bubble
    }
    fn on_moved(&mut self, _cx: &mut LifecycleContext) {}

    fn style(&self) -> &Style;

    fn paint_style(&self, cx: &mut PaintContext) {
        let style = self.style();

        for shadow in &style.paint.shadows {
            if !shadow.inset {
                cx.box_shadow(cx.bounds, shadow.clone());
            }
        }

        // Background
        cx.fill_rect(cx.bounds, style.box_model.border.radius, style.paint.background.clone());

        // Inset shadows (above background, below border)
        for shadow in &style.paint.shadows {
            if shadow.inset {
                cx.box_shadow(cx.bounds, shadow.clone());
            }
        }

        // Border
        cx.stroke_rect(cx.bounds, style.box_model.border.radius, style.box_model.border.clone());
    }

    fn paint(&self, cx: &mut PaintContext) {
        self.paint_style(cx);
    }

    fn measure(&self) -> Option<MeasureFunc> { None }

    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub trait CloneWidget {
    fn clone_box(&self) -> Box<dyn Widget>;
}

impl<W: Widget + Clone> CloneWidget for W {
    fn clone_box(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

impl<W: Widget + 'static> From<W> for Box<dyn Widget> {
    fn from(w: W) -> Self {
        Box::new(w)
    }
}

downcast_rs::impl_downcast!(Widget);

pub struct WidgetNode {
    pub(crate) widget: Box<dyn Widget>,
    pub(crate) key:    Option<WidgetKey>,
}

impl WidgetNode {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self { widget: Box::new(widget), key: None }
    }

    pub fn from_box(widget: Box<dyn Widget>) -> Self {
        Self { widget, key: None}
    }

    pub fn with_key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl<W: Widget + 'static> From<W> for WidgetNode {
    fn from(w: W) -> Self { WidgetNode::new(w) }
}