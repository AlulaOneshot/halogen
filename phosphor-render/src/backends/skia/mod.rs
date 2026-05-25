use phosphor_core::style::{BorderStyle, BoxShadow, Corners, Rect, Texture, ViewportSize};
use phosphor_core::widget::PaintSink;
use crate::backends::skia::convert::{rrect_to_sk, texture_to_paint};
use crate::PhosphorRenderer;

mod convert;

pub struct SkiaRenderer {
    surface: skia_safe::Surface,
    width: i32,
    height: i32,
}

impl SkiaRenderer {
    /// Create a CPU-raster offscreen surface.
    ///
    /// Uses `RGBAF32` pixels with a linear sRGB color space for HDR-correct
    /// compositing.
    ///
    /// Returns `None` if Skia fails to allocate the surface.
    pub fn new_offscreen(width: i32, height: i32) -> Option<Self> {
        let image_info = skia_safe::ImageInfo::new(
            (width, height),
            skia_safe::ColorType::RGBAF32,
            skia_safe::AlphaType::Premul,
            skia_safe::ColorSpace::new_srgb_linear(),
        );
        let surface = skia_safe::surfaces::raster(&image_info, None, None)?;
        Some(Self {
            surface,
            width,
            height,
        })
    }

    /// Snapshot the current surface contents as an `Image`.
    ///
    /// Useful for saving the output of a render pass to disk or comparing in tests.
    pub fn snapshot(&mut self) -> skia_safe::Image {
        self.surface.image_snapshot()
    }

    fn canvas(&mut self) -> &skia_safe::Canvas {
        self.surface.canvas()
    }
}

impl PhosphorRenderer for SkiaRenderer {
    fn begin_frame(&mut self) {
        self.surface
            .canvas().clear(skia_safe::Color::BLACK);
    }

    fn end_frame(&mut self) {
        // noop for CPU right now
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        let image_info = skia_safe::ImageInfo::new(
            (width, height),
            skia_safe::ColorType::RGBAF32,
            skia_safe::AlphaType::Premul,
            skia_safe::ColorSpace::new_srgb_linear(),
        );
        // Create a new surface at the new size
        if let Some(new_surface) = skia_safe::surfaces::raster(&image_info, None, None) {
            self.surface = new_surface;
        }
    }

    fn viewport(&self) -> ViewportSize {
        ViewportSize {
            width: self.width as f32,
            height: self.height as f32,
        }
    }
}

impl PaintSink for SkiaRenderer {
    fn draw_shadow(&mut self, rect: Rect, radii: Corners<f32>, shadow: &BoxShadow) {
        todo!();
        // let canvas = self.surface.canvas();
        //
        // if shadow.inset {
        //     draw_inset_shadow(canvas, rect, radii, shadow);
        // } else {
        //     draw_drop_shadow(canvas, rect, radii, shadow);
        // }
    }

    fn fill_rrect(&mut self, rect: Rect, radii: Corners<f32>, texture: &Texture) {
        let canvas = self.surface.canvas();
        let rrect = rrect_to_sk(rect, radii);
        let paint = texture_to_paint(texture, rect, None);
        canvas.draw_rrect(rrect, &paint);
    }

    fn stroke_rrect(&mut self, rect: Rect, radii: Corners<f32>, border: &BorderStyle) {
        todo!()
    }

    fn push_clip_rrect(&mut self, rect: Rect, radii: Corners<f32>) {
        let canvas = self.surface.canvas();
        let rrect = rrect_to_sk(rect, radii);
        canvas.save();
        canvas.clip_rrect(rrect, skia_safe::ClipOp::Intersect, true);
    }

    fn pop_clip(&mut self) {
        self.surface.canvas().restore();
    }

    fn push_layer(&mut self, opacity: f32) {
        let canvas = self.surface.canvas();
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        canvas.save_layer_alpha(None, alpha.into());
    }

    fn pop_layer(&mut self) {
        self.surface.canvas().restore();
    }
}