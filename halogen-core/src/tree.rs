use crate::geometry::{Axes, Point, Rect};
use crate::style::ViewportSize;
use crate::widget::{BuildContext, DirtyNotify, EventContext, LifecycleContext, PaintSink, Widget, WidgetId, WidgetKey, WidgetNode};
use slotmap::SlotMap;
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use crate::event::{Event, EventResult};
use crate::paint::PaintContext;
use crate::theme::Theme;

pub trait Scheduler: Send + Sync {
    fn spawn(&self, fut: Box<dyn Future<Output=()> + Send + 'static>);
    fn mark_dirty(&self, id: WidgetId);
}

pub(crate) struct RenderedNode {
    pub id: WidgetId,
    pub type_id: TypeId,
    pub key: Option<WidgetKey>,
    pub widget: Box<dyn crate::widget::Widget>,
    pub children: Vec<RenderedNode>,
    pub layout: taffy::NodeId,
    pub bounds: Rect,
    pub position: usize,
}

enum Effect {
    Mount(WidgetId),
    Unmount(WidgetId, Box<dyn Widget>),
    Moved(WidgetId),
}

struct WidgetDirtyNotify {
    is_dirty: Arc<std::sync::Mutex<bool>>,
}

impl DirtyNotify for WidgetDirtyNotify {
    fn mark_dirty(&self) {
        *self.is_dirty.lock().unwrap() = true;
    }
}

pub struct WidgetTree {
    root:      Option<RenderedNode>,
    notifiers: SlotMap<WidgetId, Arc<dyn DirtyNotify>>,
    is_dirty:  Arc<std::sync::Mutex<bool>>,
    taffy:     taffy::TaffyTree,
    viewport:  ViewportSize,
    theme:     Theme,
    scheduler: Arc<dyn Scheduler>,
}

impl WidgetTree {
    pub fn new(viewport: ViewportSize, theme: Theme, scheduler: Arc<dyn Scheduler>) -> Self {
        Self {
            root:      None,
            notifiers: SlotMap::with_key(),
            is_dirty:  Arc::new(std::sync::Mutex::new(false)),
            taffy:     taffy::TaffyTree::new(),
            viewport,
            theme,
            scheduler,
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    pub fn set_root(&mut self, node: WidgetNode) {
        let old = self.root.take();
        let mut effects = Vec::new();
        self.root = Some(self.reconcile(node, old, 0, &mut effects));
        self.commit(effects);
        self.layout();
    }

    pub fn update(&mut self) {
        let dirty = {
            let mut lock = self.is_dirty.lock().unwrap();
            let was = *lock;
            *lock = false;
            was
        };

        if !dirty { return; }

        if let Some(mut root) = self.root.take() {
            let cx_notify = Arc::downgrade(&self.notifiers[root.id]);
            let mut cx = BuildContext {
                id:           root.id,
                theme:        &self.theme,
                viewport:     self.viewport,
                dirty_notify: cx_notify,
            };

            let new_child_nodes = root.widget.build(&mut cx);
            let old_children = std::mem::take(&mut root.children);
            let mut effects = Vec::new();
            root.children = self.reconcile_children(new_child_nodes, old_children, &mut effects);

            self.taffy.set_children(
                root.layout,
                &root.children.iter().map(|c| c.layout).collect::<Vec<_>>(),
            ).unwrap();

            self.root = Some(root);
            self.commit(effects);
        }

        self.layout();
    }

    pub fn set_viewport(&mut self, viewport: ViewportSize) {
        self.viewport = viewport;
        *self.is_dirty.lock().unwrap() = true;
    }

    pub fn paint(&self, renderer: &mut dyn PaintSink) {
        if let Some(root) = &self.root {
            paint_node(root, &self.theme, renderer);
        }
    }

    pub fn on_event(&mut self, event: &Event) {
        if let Some(root) = &mut self.root {
            dispatch_event(root, event, &self.theme, &*self.scheduler);
        }
    }

    // ── Reconciliation ────────────────────────────────────────────────────────

    fn reconcile(
        &mut self,
        new:      WidgetNode,
        old:      Option<RenderedNode>,
        position: usize,
        effects:  &mut Vec<Effect>,
    ) -> RenderedNode {
        let type_id = new.widget.as_any().type_id();

        let (id, old_layout, old_children) = match old {
            Some(o) => {
                if o.position != position {
                    effects.push(Effect::Moved(o.id));
                }
                (o.id, Some(o.layout), o.children)
            }
            None => {
                let notify: Arc<dyn DirtyNotify> = Arc::new(WidgetDirtyNotify {
                    is_dirty: Arc::clone(&self.is_dirty),
                });
                let id = self.notifiers.insert(notify);
                effects.push(Effect::Mount(id));
                (id, None, vec![])
            }
        };

        let cx_notify = Arc::downgrade(&self.notifiers[id]);
        let mut cx = BuildContext {
            id,
            theme:        &self.theme,
            viewport:     self.viewport,
            dirty_notify: cx_notify,
        };

        let child_nodes = new.widget.build(&mut cx);

        let children = self.reconcile_children(child_nodes, old_children, effects);

        let layout = match old_layout {
            Some(existing) => {
                self.taffy.set_style(existing, new.widget.style().to_taffy(self.viewport)).unwrap();
                existing
            }
            None => self.taffy.new_leaf(new.widget.style().to_taffy(self.viewport)).unwrap(),
        };

        let child_layout_ids: Vec<taffy::NodeId> = children
            .iter()
            .map(|c| c.layout)
            .collect();

        self.taffy.set_children(layout, &child_layout_ids).unwrap();

        RenderedNode {
            id,
            type_id,
            key:      new.key,
            widget:   new.widget,
            children,
            layout,
            bounds:   Rect::zero(),
            position,
        }
    }

    fn reconcile_children(
        &mut self,
        new_nodes:    Vec<WidgetNode>,
        old_children: Vec<RenderedNode>,
        effects:      &mut Vec<Effect>,
    ) -> Vec<RenderedNode> {
        let mut old_keyed:   HashMap<WidgetKey, RenderedNode> = HashMap::new();
        let mut old_by_type: HashMap<TypeId, VecDeque<RenderedNode>> = HashMap::new();

        for old in old_children {
            match old.key.clone() {
                Some(k) => { old_keyed.insert(k, old); }
                None    => { old_by_type.entry(old.type_id).or_default().push_back(old); }
            }
        }

        let mut result = Vec::with_capacity(new_nodes.len());

        for (i, new_node) in new_nodes.into_iter().enumerate() {
            let type_id = new_node.widget.as_any().type_id();

            let matched = match &new_node.key {
                Some(k) => old_keyed.remove(k),
                None    => old_by_type
                    .get_mut(&type_id)
                    .and_then(|q| q.pop_front()),
            };

            result.push(self.reconcile(new_node, matched, i, effects));
        }

        for old in old_keyed.into_values()
            .chain(old_by_type.into_values().flatten())
        {
            self.unmount_subtree(old, effects);
        }

        result
    }

    fn unmount_subtree(&mut self, node: RenderedNode, effects: &mut Vec<Effect>) {
        for child in node.children {
            self.unmount_subtree(child, effects);
        }
        self.taffy.remove(node.layout).ok();
        self.notifiers.remove(node.id);
        effects.push(Effect::Unmount(node.id, node.widget));
    }

    // ── Commit ────────────────────────────────────────────────────────────────

    fn commit(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Mount(id) => {
                    let mut cx = LifecycleContext {
                        id,
                        theme:     &self.theme,
                        viewport:  self.viewport,
                        scheduler: &*self.scheduler,
                    };
                    // Widget is now in self.root — walk to find it and call on_mount
                    if let Some(node) = find_node_mut(self.root.as_mut(), id) {
                        node.widget.on_mount(&mut cx);
                    }
                }
                Effect::Unmount(id, mut widget) => {
                    let mut cx = LifecycleContext {
                        id,
                        theme:     &self.theme,
                        viewport:  self.viewport,
                        scheduler: &*self.scheduler,
                    };
                    widget.on_unmount(&mut cx);
                }
                Effect::Moved(id) => {
                    let mut cx = LifecycleContext {
                        id,
                        theme:     &self.theme,
                        viewport:  self.viewport,
                        scheduler: &*self.scheduler,
                    };
                    if let Some(node) = find_node_mut(self.root.as_mut(), id) {
                        node.widget.on_moved(&mut cx);
                    }
                }
            }
        }
    }

    fn layout(&mut self) {
        let Some(root) = &self.root else { return; };

        // Tell taffy the available space is the viewport
        let available = taffy::Size {
            width:  taffy::AvailableSpace::Definite(self.viewport.width.into()),
            height: taffy::AvailableSpace::Definite(self.viewport.height.into()),
        };

        self.taffy.compute_layout(root.layout, available).unwrap();

        // Walk the tree and write resolved bounds back into each RenderedNode
        if let Some(root) = self.root.as_mut() {
            write_bounds(root, Rect::zero(), &self.taffy);
        }
    }
}

fn write_bounds(node: &mut RenderedNode, parent: Rect, taffy: &taffy::TaffyTree) {
    let layout = taffy.layout(node.layout).unwrap();

    // Taffy gives us position relative to parent — accumulate absolute origin
    node.bounds = Rect {
        origin: Point::new(parent.origin.x + layout.location.x, parent.origin.y + layout.location.y),
        size: Axes::new(layout.size.width, layout.size.height)
    };

    for child in &mut node.children {
        write_bounds(child, node.bounds, taffy);
    }
}

// ── Tree walk helpers ─────────────────────────────────────────────────────────

fn find_node_mut(node: Option<&mut RenderedNode>, id: WidgetId) -> Option<&mut RenderedNode> {
    let node = node?;
    if node.id == id { return Some(node); }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(Some(child), id) {
            return Some(found);
        }
    }
    None
}

fn paint_node(node: &RenderedNode, theme: &Theme, renderer: &mut dyn PaintSink) {
    dbg!(node.widget.debug_name(), node.bounds);
    let mut cx = PaintContext { bounds: node.bounds, theme, renderer };
    node.widget.paint(&mut cx);
    for child in &node.children { paint_node(child, theme, renderer); }
}

fn dispatch_event(
    node:      &mut RenderedNode,
    event:     &Event,
    theme:     &Theme,
    scheduler: &dyn Scheduler,
) -> EventResult {
    for child in &mut node.children {
        if let EventResult::Consumed = dispatch_event(child, event, theme, scheduler) {
            return EventResult::Consumed;
        }
    }
    let mut cx = EventContext { id: node.id, bounds: node.bounds, theme, scheduler };
    node.widget.on_event(event, &mut cx)
}