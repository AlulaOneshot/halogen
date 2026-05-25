//! Rendering backends for Phosphor.
//!
//! This crate provides implementations of [`PaintSink`](phosphor_core::widget::PaintSink)
//! for driving the widget tree's paint pass.
//!
//! ## Backends
//!
//! | Feature | Type | Use |
//! |---|---|---|
//! | `skia` (default) | [`SkiaRenderer`] | Production — GPU via OpenGL ES, CPU raster fallback |
//! | `headless` | [`HeadlessRenderer`] | Tests — records draw calls without rendering |
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut renderer = SkiaRenderer::new_offscreen(1280, 720).unwrap();
//! let mut tree = WidgetTree::new(viewport, theme);
//! tree.set_root(my_widget);
//!
//! // Each frame:
//! tree.rebuild();
//! tree.layout();
//! renderer.begin_frame();
//! tree.paint(&mut renderer);
//! renderer.end_frame();
//! ```

pub mod renderer;

pub mod backends;

pub use renderer::PhosphorRenderer;

#[cfg(feature = "skia")]
pub use backends::skia::SkiaRenderer;

pub use skia_safe as skia;

#[cfg(feature = "headless")]
pub use backends::headless::HeadlessRenderer;