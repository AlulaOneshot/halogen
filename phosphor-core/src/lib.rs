//! # phosphor-core
//!
//! Core runtime for the Phosphor UI framework.
//!
//! Phosphor is a signal-driven, retained-mode UI framework designed for TV and console interfaces, mainly targeting Raspberry Pi 5 at 1920x1080@60.
//! This crate covers the foundations: widget model, styling, reactive state, event model, and layout.
//! It should carry ***no renderer dependencies*** and ***no async runtime***.
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! |[`widget`]|[`widget::Widget`] trait, build/paint/event contexts, and [`widget::WidgetNode`]|
//! |[`style`]|[`style::Style`] trait, [`style::WidgetStyle`] builder, layout types.
//! |[`color`]|[`color::Color`] for coloring, and predefined colors in [`color::Colors`]
//! |[`theme`]|[`theme::Theme`] for a type based theming map|
//! |[`event`]|[`event::Event<T>`] for an extensible event system|
//! |[`signal`]|[`signal::Signal<T>`] for reactive state with auto-subscription|
//! |[`handler`]|[`handler::Handler`] for sync/async callbacks on events|
//! |[`tree`]|[`WidgetTree`] for mounting, layout, paint, and dispatch of events|

pub mod color;
pub mod event;
pub mod handler;
pub mod signal;
pub mod style;
pub mod theme;
pub mod tree;
pub mod widget;
