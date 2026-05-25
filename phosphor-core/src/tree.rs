//! The widget tree: mount, reconcile, layout, paint, and event dispatch.
//!
//! [`WidgetTree`] is the central runtime object. It owns all live widget instances,
//! drives the rebuild/layout/paint cycle, and routes input events.
//!
//! ## Cycle
//!
//! 1. **[`rebuild`](WidgetTree::rebuild)** - drain the dirty queue; call `build()` on
//!    dirty widgets depth-first (parents before children); reconcile new children
//!    against existing ones.
//! 2. **[`layout`](WidgetTree::layout)** - run Taffy on the full tree; resolve
//!    absolute screen bounds.
//! 3. **[`paint`](WidgetTree::paint)** - walk the tree depth-first, calling each
//!    widget's `paint()`.
//! 4. **[`dispatch`](WidgetTree::dispatch)** - route an event depth-first (deepest
//!    widget first); stop when [`EventResult::Consumed`] is returned.
//!
//! ## Reconciliation
//!
//! When a widget's `build()` returns a new child list, the tree diffs it against
//! the existing children using a two-pass strategy:
//!
//! - **Keyed nodes** are matched by key string across positions.
//! - **Unkeyed nodes** are matched positionally.
//!
//! If a matched node has the same concrete type, the widget instance is kept in
//! place and marked dirty (update-in-place). If the type differs, the old subtree
//! is unmounted and a fresh one is mounted.
//!
//! ## Dirty tracking
//!
//! Each widget has a `Weak<WidgetDirtyNotify>` given to any signal it reads during
//! `build()`. When a signal fires, it calls `mark_dirty` through the weak pointer,
//! inserting the widget's `WidgetId` into `dirty_queue`. Dead `Weak` pointers
//! (unmounted widgets) are pruned automatically by the signal.

use crate::event::{Event, EventResult};
use crate::handler::HandlerExecutor;
use crate::signal::DirtyNotify;
use crate::style::{Rect, ViewportSize};
use crate::theme::Theme;
use crate::widget::{
    BuildContext, EventContext, PaintContext, PaintSink, Widget, WidgetId, WidgetNode,
};
use parking_lot::Mutex;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak};

struct TreeEntry {
    widget: Box<dyn Widget>,
    children: Vec<WidgetId>,
    parent: Option<WidgetId>,
    key: Option<String>,
    layout_node: taffy::NodeId,
    bounds: Rect,
    dirty: bool,
}

struct WidgetDirtyNotify {
    id: WidgetId,
    dirty_queue: Arc<Mutex<HashSet<WidgetId>>>,
}

impl DirtyNotify for WidgetDirtyNotify {
    fn mark_dirty(&self) {
        self.dirty_queue.lock().insert(self.id);
    }
}

/// The live widget tree.
///
/// Create with [`WidgetTree::new`], set the root widget with [`set_root`](WidgetTree::set_root),
/// then drive the rebuild/layout/paint cycle each frame.
pub struct WidgetTree {
    nodes: SlotMap<WidgetId, TreeEntry>,
    root: Option<WidgetId>,
    notifiers: HashMap<WidgetId, Arc<WidgetDirtyNotify>>,
    dirty_queue: Arc<Mutex<HashSet<WidgetId>>>,
    taffy: taffy::TaffyTree,
    viewport: ViewportSize,
    theme: Theme,
}

impl WidgetTree {
    /// Create an empty tree with the given viewport size and theme.
    pub fn new(viewport: ViewportSize, theme: Theme) -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
            notifiers: HashMap::new(),
            dirty_queue: Arc::new(Mutex::new(HashSet::new())),
            taffy: taffy::TaffyTree::new(),
            viewport,
            theme,
        }
    }

    /// Set the root widget, replacing the previous root and unmounting its entire subtree.
    pub fn set_root(&mut self, widget: impl Widget + 'static) {
        if let Some(old_root) = self.root.take() {
            self.remove_subtree(old_root);
        }
        self.root = Some(self.mount(WidgetNode::new(widget), None));
    }

    fn mount(&mut self, node: WidgetNode, parent: Option<WidgetId>) -> WidgetId {
        let layout_node = self
            .taffy
            .new_leaf(taffy::Style::default())
            .expect("taffy new_leaf");

        let id = self.nodes.insert_with_key(|_id| TreeEntry {
            widget: node.widget,
            children: Vec::new(),
            parent,
            key: node.key,
            layout_node,
            bounds: Rect::default(),
            dirty: true, // always rebuild on first mount
        });

        self.notifiers.insert(
            id,
            Arc::new(WidgetDirtyNotify {
                id,
                dirty_queue: Arc::clone(&self.dirty_queue),
            }),
        );

        id
    }

    fn remove_subtree(&mut self, id: WidgetId) {
        let children: Vec<WidgetId> = self
            .nodes
            .get(id)
            .map(|e| e.children.clone())
            .unwrap_or_default();

        for child in children {
            self.remove_subtree(child);
        }

        if let Some(entry) = self.nodes.remove(id) {
            let _ = self.taffy.remove(entry.layout_node);
        }
        self.notifiers.remove(&id);
        self.dirty_queue.lock().remove(&id);
    }

    /// Run the rebuild pass.
    ///
    /// Drains the dirty queue and rebuilds any dirty widgets depth-first.
    /// Returns `true` if any widgets were rebuilt (indicating layout + paint are needed).
    pub fn rebuild(&mut self) -> bool {
        let dirty: HashSet<WidgetId> = self.dirty_queue.lock().drain().collect();
        let has_dirty_in_tree = self.nodes.values().any(|e| e.dirty);

        if dirty.is_empty() && !has_dirty_in_tree {
            return false;
        }

        if let Some(root) = self.root {
            self.rebuild_subtree(root, &dirty);
        }

        true
    }

    fn rebuild_subtree(&mut self, id: WidgetId, dirty: &HashSet<WidgetId>) {
        let is_dirty = self.nodes.get(id).map(|e| e.dirty).unwrap_or(false) || dirty.contains(&id);

        if is_dirty {
            self.rebuild_one(id);
        }

        // Re-read children after rebuild - parent may have changed them.
        let children: Vec<WidgetId> = self
            .nodes
            .get(id)
            .map(|e| e.children.clone())
            .unwrap_or_default();

        for child in children {
            self.rebuild_subtree(child, dirty);
        }
    }

    fn rebuild_one(&mut self, id: WidgetId) {
        let notifier = match self.notifiers.get(&id) {
            Some(n) => Arc::downgrade(n) as Weak<dyn DirtyNotify>,
            None => return,
        };

        if let Some(entry) = self.nodes.get_mut(id) {
            entry.dirty = false;
        }

        let new_children = {
            let entry = match self.nodes.get(id) {
                Some(e) => e,
                None => return,
            };
            let mut cx = BuildContext {
                widget_id: id,
                theme: &self.theme,
                dirty_notify: notifier,
                viewport: self.viewport,
            };
            // SAFETY: `build()` receives `&mut BuildContext` which does not alias
            // `entry.widget`. The widget is read through a raw pointer to break the
            // exclusive borrow on `self.nodes` while still passing `&mut cx` (which
            // borrows `self.theme` and `self.viewport`, not `self.nodes`).
            // The widget is not written to during `build()` - only `cx` is mutated.
            let widget = unsafe { &*(entry.widget.as_ref() as *const dyn Widget) };
            widget.build(&mut cx)
        };

        // Update Taffy layout style from the rebuilt widget.
        let layout_style = self
            .nodes
            .get(id)
            .map(|e| e.widget.layout_style(self.viewport))
            .unwrap_or_default();

        if let Some(entry) = self.nodes.get(id) {
            let _ = self.taffy.set_style(entry.layout_node, layout_style);
        }

        self.reconcile(id, new_children);
    }

    fn reconcile(&mut self, parent_id: WidgetId, new_nodes: Vec<WidgetNode>) {
        let old_children: Vec<WidgetId> = self
            .nodes
            .get(parent_id)
            .map(|e| e.children.clone())
            .unwrap_or_default();

        // Build key → old_id map for O(1) keyed lookups.
        let mut keyed: HashMap<String, WidgetId> = old_children
            .iter()
            .filter_map(|&id| {
                self.nodes
                    .get(id)
                    .and_then(|e| e.key.clone())
                    .map(|k| (k, id))
            })
            .collect();

        let mut matched: HashSet<WidgetId> = HashSet::new();

        // Unkeyed old children for positional fallback.
        let positional: Vec<WidgetId> = old_children
            .iter()
            .filter(|&&id| self.nodes.get(id).and_then(|e| e.key.as_ref()).is_none())
            .copied()
            .collect();
        let mut pos_idx = 0;

        let mut final_children: Vec<WidgetId> = Vec::with_capacity(new_nodes.len());

        for new_node in new_nodes {
            let matched_id = if let Some(ref key) = new_node.key {
                keyed.remove(key).map(|old_id| {
                    matched.insert(old_id);
                    old_id
                })
            } else {
                // Skip already-matched positional entries.
                while pos_idx < positional.len() && matched.contains(&positional[pos_idx]) {
                    pos_idx += 1;
                }
                if pos_idx < positional.len() {
                    let old_id = positional[pos_idx];
                    matched.insert(old_id);
                    pos_idx += 1;
                    Some(old_id)
                } else {
                    None
                }
            };

            let child_id = match matched_id {
                Some(old_id) => {
                    let same_type = self
                        .nodes
                        .get(old_id)
                        .map(|e| e.widget.as_ref().type_id() == new_node.widget.as_ref().type_id())
                        .unwrap_or(false);

                    if same_type {
                        // Same type: update widget data and schedule rebuild.
                        if let Some(entry) = self.nodes.get_mut(old_id) {
                            entry.widget = new_node.widget;
                            entry.key = new_node.key;
                            entry.dirty = true;
                        }
                        self.dirty_queue.lock().insert(old_id);
                        old_id
                    } else {
                        // Type changed: unmount the old subtree, mount fresh.
                        self.remove_subtree(old_id);
                        self.mount(new_node, Some(parent_id))
                    }
                }
                None => self.mount(new_node, Some(parent_id)),
            };

            final_children.push(child_id);
        }

        // Unmount old children that were not matched.
        for &old_id in &old_children {
            if !matched.contains(&old_id) && !final_children.contains(&old_id) {
                self.remove_subtree(old_id);
            }
        }

        // Update parent's child list.
        if let Some(entry) = self.nodes.get_mut(parent_id) {
            entry.children = final_children.clone();
        }

        // Sync Taffy's child list to match.
        if let Some(parent_layout) = self.nodes.get(parent_id).map(|e| e.layout_node) {
            let child_layouts: Vec<taffy::NodeId> = final_children
                .iter()
                .filter_map(|id| self.nodes.get(*id).map(|e| e.layout_node))
                .collect();
            let _ = self.taffy.set_children(parent_layout, &child_layouts);
        }
    }

    /// Run Taffy layout and resolve absolute screen bounds for every widget.
    ///
    /// Must be called after [`rebuild`](WidgetTree::rebuild) and before
    /// [`paint`](WidgetTree::paint).
    pub fn layout(&mut self) {
        let root_layout = match self.root.and_then(|r| self.nodes.get(r)) {
            Some(e) => e.layout_node,
            None => return,
        };

        self.taffy
            .compute_layout(
                root_layout,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(self.viewport.width),
                    height: taffy::AvailableSpace::Definite(self.viewport.height),
                },
            )
            .expect("taffy compute_layout");

        if let Some(root) = self.root {
            self.resolve_bounds(root, crate::style::Point { x: 0.0, y: 0.0 });
        }
    }

    fn resolve_bounds(&mut self, id: WidgetId, parent_origin: crate::style::Point) {
        let layout = {
            let entry = match self.nodes.get(id) {
                Some(e) => e,
                None => return,
            };
            *self.taffy.layout(entry.layout_node).expect("taffy layout")
        };

        let origin = crate::style::Point {
            x: parent_origin.x + layout.location.x,
            y: parent_origin.y + layout.location.y,
        };

        let children = self
            .nodes
            .get_mut(id)
            .map(|e| {
                e.bounds = Rect {
                    x: origin.x,
                    y: origin.y,
                    width: layout.size.width,
                    height: layout.size.height,
                };
                e.children.clone()
            })
            .unwrap_or_default();

        for child in children {
            self.resolve_bounds(child, origin);
        }
    }

    /// Update the viewport size. Does not trigger a layout pass - call
    /// [`layout`](WidgetTree::layout) after this.
    pub fn set_viewport(&mut self, viewport: ViewportSize) {
        self.viewport = viewport;
    }

    /// Walk the tree depth-first, calling each widget's `paint()`.
    ///
    /// Must be called after [`layout`](WidgetTree::layout).
    pub fn paint(&self, sink: &mut dyn PaintSink) {
        if let Some(root) = self.root {
            self.paint_subtree(root, sink);
        }
    }

    fn paint_subtree(&self, id: WidgetId, sink: &mut dyn PaintSink) {
        let entry = match self.nodes.get(id) {
            Some(e) => e,
            None => return,
        };
        let mut cx = PaintContext {
            bounds: entry.bounds,
            theme: &self.theme,
            renderer: sink,
        };
        entry.widget.paint(&mut cx);
        for &child in &entry.children {
            self.paint_subtree(child, sink);
        }
    }

    /// Dispatch an event through the tree depth-first (deepest widget first).
    ///
    /// Returns [`EventResult::Consumed`] if any widget handled the event,
    /// [`EventResult::Bubble`] if no widget consumed it.
    ///
    /// [`Event::Custom`] variants are not currently routed to individual widgets.
    pub fn dispatch<E>(&self, event: &Event<E>, executor: &dyn HandlerExecutor) -> EventResult
    where
        E: Clone + std::fmt::Debug + PartialEq,
    {
        self.root
            .map(|root| self.dispatch_subtree(root, event, executor))
            .unwrap_or(EventResult::Bubble)
    }

    fn dispatch_subtree<E>(
        &self,
        id: WidgetId,
        event: &Event<E>,
        executor: &dyn HandlerExecutor,
    ) -> EventResult
    where
        E: Clone + std::fmt::Debug + PartialEq,
    {
        let entry = match self.nodes.get(id) {
            Some(e) => e,
            None => return EventResult::Bubble,
        };

        // Children first - deepest widget wins.
        for &child in entry.children.iter().rev() {
            if self.dispatch_subtree(child, event, executor) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }

        // Map Event<E> → Event<()>. Custom events are stripped here.
        let base_event: Option<Event<()>> = match event {
            Event::KeyDown(e) => Some(Event::KeyDown(e.clone())),
            Event::KeyUp(e) => Some(Event::KeyUp(e.clone())),
            Event::TextInput(s) => Some(Event::TextInput(s.clone())),
            Event::GamepadButtonDown(e) => Some(Event::GamepadButtonDown(*e)),
            Event::GamepadButtonUp(e) => Some(Event::GamepadButtonUp(*e)),
            Event::GamepadAxisChanged(e) => Some(Event::GamepadAxisChanged(*e)),
            Event::GamepadConnected(id) => Some(Event::GamepadConnected(*id)),
            Event::GamepadDisconnected(id) => Some(Event::GamepadDisconnected(*id)),
            Event::Focus(e) => Some(Event::Focus(*e)),
            Event::Lifecycle(e) => Some(Event::Lifecycle(*e)),
            Event::Custom(_) => None, // not yet routed to widgets
        };

        if let Some(ev) = base_event {
            let mut cx = EventContext {
                widget_id: id,
                bounds: entry.bounds,
                theme: &self.theme,
                executor,
            };
            entry.widget.on_event(&ev, &mut cx)
        } else {
            EventResult::Bubble
        }
    }

    /// Replace the active theme and mark every widget dirty.
    ///
    /// Triggers a full rebuild on the next [`rebuild`](WidgetTree::rebuild) call,
    /// since all widgets that read theme tokens need to re-evaluate.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        for (id, _) in &self.nodes {
            self.dirty_queue.lock().insert(id);
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Total number of mounted widget instances.
    pub fn widget_count(&self) -> usize {
        self.nodes.len()
    }

    /// The root widget's `WidgetId`, if a root has been set.
    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    /// The resolved screen bounds of the widget with the given ID, if it exists.
    pub fn bounds_of(&self, id: WidgetId) -> Option<Rect> {
        self.nodes.get(id).map(|e| e.bounds)
    }
}
