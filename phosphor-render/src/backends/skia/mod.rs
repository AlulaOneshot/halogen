use phosphor_core::style::{BorderStyle, BoxShadow, Corners, Rect, ShadowSizing, Texture, ViewportSize};
use phosphor_core::widget::PaintSink;
use crate::backends::skia::convert::{color_to_sk, expand_rect, offset_rect, rect_to_sk, rrect_to_sk, texture_to_paint};
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
        let width = self.surface.width() as f32;
        let height = self.surface.height() as f32;

        let canvas = self.surface.canvas();

        if shadow.inset {
            draw_inset_shadow(canvas, rect, radii, shadow, ViewportSize {
                width,
                height,
            });
        } else {
            draw_drop_shadow(canvas, rect, radii, shadow, ViewportSize {
                width,
                height,
            });
        }
    }

    fn fill_rrect(&mut self, rect: Rect, radii: Corners<f32>, texture: &Texture) {
        let canvas = self.surface.canvas();
        let rrect = rrect_to_sk(rect, radii);
        let paint = texture_to_paint(texture, rect, None);
        canvas.draw_rrect(rrect, &paint);
    }

    fn stroke_rrect(&mut self, rect: Rect, radii: Corners<f32>, border: &BorderStyle) {
        let canvas = self.surface.canvas();

        // Check if all edges are uniform (same width and texture).
        let e = &border.edges;
        let uniform_width = e.top.width == e.right.width
            && e.right.width == e.bottom.width
            && e.bottom.width == e.left.width;
        let uniform_texture = e.top.texture == e.right.texture
            && e.right.texture == e.bottom.texture
            && e.bottom.texture == e.left.texture;

        if uniform_width && uniform_texture {
            // Fast path: single stroked rrect.
            let rrect = rrect_to_sk(rect, radii);
            let paint = texture_to_paint(&e.top.texture, rect, Some(e.top.width));
            canvas.draw_rrect(rrect, &paint);
        } else {
            // Per-edge path: clip each edge region and fill it.
            // Corners belong to the horizontal edge (CSS convention).
            draw_per_edge_border(canvas, rect, radii, border);
        }
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

fn draw_drop_shadow(canvas: &skia_safe::Canvas, rect: Rect, radii: Corners<f32>, shadow: &BoxShadow, viewport_size: ViewportSize) {
    let compiled_shadow_size = {
        let x = match shadow.offset.0 {
            ShadowSizing::Pixels(px) => px,
            ShadowSizing::ScaleUnits(s) => s.0 * (viewport_size.width / 1280.0)
        };
        let y = match shadow.offset.1 {
            ShadowSizing::Pixels(px) => px,
            ShadowSizing::ScaleUnits(s) => s.0 * (viewport_size.height / 1280.0)
        };
        (x, y)
    };

    // Expand by spread, then offset.
    let shadow_rect = offset_rect(expand_rect(rect, shadow.spread), compiled_shadow_size);
    let rrect = rrect_to_sk(shadow_rect, radii);

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(
        color_to_sk(shadow.color),
        &skia_safe::ColorSpace::new_srgb_linear(),
    );
    // Sigma: CSS blur radius is the Gaussian standard deviation.
    if shadow.blur > 0.0 {
        paint.set_mask_filter(skia_safe::MaskFilter::blur(
            skia_safe::BlurStyle::Normal,
            shadow.blur,
            false,
        ));
    }

    canvas.draw_rrect(rrect, &paint);
}

fn draw_inset_shadow(canvas: &skia_safe::Canvas, rect: Rect, radii: Corners<f32>, shadow: &BoxShadow, viewport_size: ViewportSize) {
    // Clip to the widget's interior, then draw a blurred rect that bleeds inward.
    let inner_rrect = rrect_to_sk(rect, radii);
    canvas.save();
    canvas.clip_rrect(inner_rrect, skia_safe::ClipOp::Intersect, true);

    let compiled_shadow_size = {
        let x = match shadow.offset.0 {
            ShadowSizing::Pixels(px) => px,
            ShadowSizing::ScaleUnits(s) => s.0 * (viewport_size.width / 1280.0)
        };
        let y = match shadow.offset.1 {
            ShadowSizing::Pixels(px) => px,
            ShadowSizing::ScaleUnits(s) => s.0 * (viewport_size.height / 1280.0)
        };
        (x, y)
    };

    // The shadow rect is the widget rect shrunk by spread and shifted by the offset.
    // We draw it as a large rect outside the original bounds so the blur bleeds in.
    // Using a rect much larger than the widget ensures full coverage of the edge.
    let margin = shadow.blur * 2.0 + shadow.spread;
    let outer = expand_rect(rect, margin);
    let inner = expand_rect(rect, -shadow.spread);
    let inner_offset = offset_rect(inner, compiled_shadow_size);

    // Build a path: large outer rect minus the offset inner rect.
    // The difference is where the shadow color is applied; blur bleeds over the edge.
    let mut path = skia_safe::PathBuilder::new();
    path.add_rect(rect_to_sk(outer), None, 0);
    path.add_rect(rect_to_sk(inner_offset), None, 0);
    // Even-odd fill rule: the inner rect punches a hole.
    path.set_fill_type(skia_safe::PathFillType::EvenOdd);
    let path = path.snapshot();

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(
        color_to_sk(shadow.color),
        &skia_safe::ColorSpace::new_srgb_linear(),
    );
    if shadow.blur > 0.0 {
        paint.set_mask_filter(skia_safe::MaskFilter::blur(
            skia_safe::BlurStyle::Normal,
            shadow.blur,
            false,
        ));
    }

    canvas.draw_path(&path, &paint);
    canvas.restore();
}

/// Draw a border where edges may have different widths or textures.
///
/// Corner attribution follows CSS: the horizontal edge (top/bottom) owns the corner.
/// Each edge is drawn by clipping to a trapezoidal region and filling.
fn draw_per_edge_border(canvas: &skia_safe::Canvas, rect: Rect, radii: Corners<f32>, border: &BorderStyle) {
    let e = &border.edges;

    // Outer and inner rects (approximate — ignores radius for inner rect).
    let top_w = e.top.width;
    let right_w = e.right.width;
    let bottom_w = e.bottom.width;
    let left_w = e.left.width;

    // For each edge we build a trapezoid path and clip to it before drawing.
    // The trapezoid covers the edge region including the adjacent corner portions.
    // Corners are attributed to the horizontal edge (CSS behavior).

    let x = rect.x;
    let y = rect.y;
    let r = rect.x + rect.width;
    let b = rect.y + rect.height;

    // Top edge: full width including corners
    if top_w > 0.0 {
        let mut path = skia_safe::PathBuilder::new();
        path.move_to((x, y));
        path.line_to((r, y));
        path.line_to((r - right_w, y + top_w));
        path.line_to((x + left_w, y + top_w));
        path.close();
        let path = path.snapshot();

        let paint = texture_to_paint(&e.top.texture, rect, None);
        canvas.save();
        canvas.clip_path(&path, skia_safe::ClipOp::Intersect, true);
        canvas.draw_rrect(rrect_to_sk(rect, radii), &paint);
        canvas.restore();
    }

    // Bottom edge: full width including corners
    if bottom_w > 0.0 {
        let mut path = skia_safe::PathBuilder::new();
        path.move_to((x, b));
        path.line_to((r, b));
        path.line_to((r - right_w, b - bottom_w));
        path.line_to((x + left_w, b - bottom_w));
        path.close();
        let path = path.snapshot();

        let paint = texture_to_paint(&e.bottom.texture, rect, None);
        canvas.save();
        canvas.clip_path(&path, skia_safe::ClipOp::Intersect, true);
        canvas.draw_rrect(rrect_to_sk(rect, radii), &paint);
        canvas.restore();
    }

    // Right edge: excludes corners (owned by top/bottom)
    if right_w > 0.0 {
        let mut path = skia_safe::PathBuilder::new();
        path.move_to((r, y + top_w));
        path.line_to((r, b - bottom_w));
        path.line_to((r - right_w, b - bottom_w));
        path.line_to((r - right_w, y + top_w));
        path.close();
        let path = path.snapshot();

        let paint = texture_to_paint(&e.right.texture, rect, None);
        canvas.save();
        canvas.clip_path(&path, skia_safe::ClipOp::Intersect, true);
        canvas.draw_rrect(rrect_to_sk(rect, radii), &paint);
        canvas.restore();
    }

    // Left edge: excludes corners (owned by top/bottom)
    if left_w > 0.0 {
        let mut path = skia_safe::PathBuilder::new();
        path.move_to((x, y + top_w));
        path.line_to((x, b - bottom_w));
        path.line_to((x + left_w, b - bottom_w));
        path.line_to((x + left_w, y + top_w));
        path.close();
        let path = path.snapshot();

        let paint = texture_to_paint(&e.left.texture, rect, None);
        canvas.save();
        canvas.clip_path(&path, skia_safe::ClipOp::Intersect, true);
        canvas.draw_rrect(rrect_to_sk(rect, radii), &paint);
        canvas.restore();
    }
}