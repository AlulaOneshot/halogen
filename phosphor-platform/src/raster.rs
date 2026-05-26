//! CPU raster backend using softbuffer for presentation.
//!
//! Renders into a Skia CPU surface, then blits the pixel data into a
//! softbuffer surface for display. No GPU required.

use super::GpuContext;
use phosphor_render::SkiaRenderer;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::window::Window;

pub struct RasterContext {
    surface:  Surface<Arc<Window>, Arc<Window>>,
    renderer: SkiaRenderer,
    width:    i32,
    height:   i32,
}

impl RasterContext {
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        let size   = window.inner_size();
        let width  = size.width  as i32;
        let height = size.height as i32;

        let context = Context::new(Arc::clone(&window))
            .map_err(|e| format!("softbuffer context: {e}"))?;
        let mut surface = Surface::new(&context, Arc::clone(&window))
            .map_err(|e| format!("softbuffer surface: {e}"))?;

        surface.resize(
            NonZeroU32::new(size.width).unwrap_or(NonZeroU32::new(1).unwrap()),
            NonZeroU32::new(size.height).unwrap_or(NonZeroU32::new(1).unwrap()),
        ).map_err(|e| format!("softbuffer resize: {e}"))?;

        let renderer = SkiaRenderer::new_offscreen(width, height)
            .ok_or("failed to create Skia offscreen surface")?;

        Ok(Self { surface, renderer, width, height })
    }
}

impl GpuContext for RasterContext {
    fn renderer(&mut self) -> &mut SkiaRenderer {
        &mut self.renderer
    }

    fn present(&mut self) {
        // Snapshot the Skia surface into raw pixels.
        let image = self.renderer.snapshot();
        let info  = skia_safe::ImageInfo::new(
            (self.width, self.height),
            skia_safe::ColorType::BGRA8888,
            skia_safe::AlphaType::Premul,
            None,
        );

        let row_bytes = self.width as usize * 4;
        let mut pixels = vec![0u32; (self.width * self.height) as usize];

        // Read pixels as BGRA8888 directly into our u32 buffer.
        // softbuffer expects 0xXXRRGGBB (native-endian BGRX), which matches BGRA8888 on LE.
        let pixel_bytes = unsafe {
            std::slice::from_raw_parts_mut(pixels.as_mut_ptr() as *mut u8, pixels.len() * 4)
        };
        image.read_pixels(&info, pixel_bytes, row_bytes, (0, 0), skia_safe::image::CachingHint::Allow);

        if let Ok(mut buf) = self.surface.buffer_mut() {
            buf.copy_from_slice(&pixels);
            let _ = buf.present();
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width  = width;
        self.height = height;

        let w = NonZeroU32::new(width  as u32).unwrap_or(NonZeroU32::new(1).unwrap());
        let h = NonZeroU32::new(height as u32).unwrap_or(NonZeroU32::new(1).unwrap());
        let _ = self.surface.resize(w, h);

        if let Some(r) = SkiaRenderer::new_offscreen(width, height) {
            self.renderer = r;
        }
    }
}