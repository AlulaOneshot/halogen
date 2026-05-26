use std::sync::Arc;
use winit::window::Window;
use phosphor_render::SkiaRenderer;

pub mod app;
pub mod event;
mod raster;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Raster
}

impl GpuBackend {
    /// Select the best available backend for the current platform.
    ///
    /// # Panics
    ///
    /// Panics if no backend feature is enabled for the current platform.
    pub fn best_for_platform() -> Self {
        GpuBackend::Raster
    }

    /// Create a [`GpuContext`] for this backend and the given window.
    ///
    /// # Errors
    ///
    /// Returns an error string if context creation fails.
    pub fn create_context(
        self,
        window: Arc<Window>,
    ) -> Result<Box<dyn GpuContext>, String> {
        match self {
            GpuBackend::Raster => raster::RasterContext::new(window).map(|ctx| Box::new(ctx) as Box<dyn GpuContext>)
        }
    }
}


pub trait GpuContext {
    /// The renderer for the current frame. Call `begin_frame` / `end_frame` around painting.
    fn renderer(&mut self) -> &mut SkiaRenderer;

    /// Present the current frame to the screen.
    fn present(&mut self);

    /// Resize the swap chain. Call when the window is resized.
    fn resize(&mut self, width: i32, height: i32);
}