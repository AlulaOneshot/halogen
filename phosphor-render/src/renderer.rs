//! The `PhosphorRenderer` trait — high-level frame lifecycle on top of `PaintSink`.

use phosphor_core::style::ViewportSize;
use phosphor_core::widget::PaintSink;

/// A renderer that handles frame lifecycle and rendering.
///
/// Implementors also implement [`PaintSink`], we only add frame lifecycle methods here.
///
/// ## Frame loop
///
/// ```rust,ignore
/// renderer.begin_frame();
/// tree.paint(&mut renderer);
/// renderer.end_frame();
/// ```
pub trait PhosphorRenderer: PaintSink {
    /// Clear surface for new frame.
    fn begin_frame(&mut self);

    /// Finish the frame. GPU backends will flush the command buffer, headless does nothing.
    fn end_frame(&mut self);

    /// Resize the render surface.
    fn resize(&mut self, width: i32, height: i32);

    /// The current surface size.
    fn viewport(&self) -> ViewportSize;
}