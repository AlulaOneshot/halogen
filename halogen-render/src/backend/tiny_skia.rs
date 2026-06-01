use crate::backend::Backend;
use halogen_core::geometry::{Corners, Rect};
use halogen_core::paint::{BlendMode, PaintCommand, Transform};
use halogen_core::style::Paint;
use std::num::NonZeroU32;
use halogen_core::color::{Color, ColorStops, Colors};

pub struct SkiaBackend<
    D: raw_window_handle::HasDisplayHandle,
    W: raw_window_handle::HasWindowHandle,
> {
    pixmap: tiny_skia::Pixmap,
    surface: softbuffer::Surface<D, W>,

    width: u32,
    height: u32,

    // Push/pop stacks
    opacity_stack: Vec<f32>,
    clip_stack: Vec<ClipEntry>,
    blend_stack: Vec<tiny_skia::BlendMode>,
    transform_stack: Vec<tiny_skia::Transform>,

    // Accumulated state
    current_opacity: f32,
    current_blend: tiny_skia::BlendMode,
    current_transform: tiny_skia::Transform,
}

#[derive(Clone)]
struct ClipEntry {
    rect: Rect,
    radius: Corners<f32>, // uniform approximation for now
}

impl<D: raw_window_handle::HasDisplayHandle, W: raw_window_handle::HasWindowHandle>
    SkiaBackend<D, W>
{
    pub fn new(display: D, window: W, width: u32, height: u32) -> Self {
        let context = softbuffer::Context::new(display).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window).unwrap();

        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        let pixmap = tiny_skia::Pixmap::new(width, height).unwrap();

        Self {
            pixmap,
            surface,

            width,
            height,

            opacity_stack: vec![],
            clip_stack: vec![],
            blend_stack: vec![],
            transform_stack: vec![],

            current_opacity: 1.0,
            current_blend: tiny_skia::BlendMode::SourceOver,
            current_transform: tiny_skia::Transform::identity(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        self.width = width;
        self.height = height;
        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();
        self.pixmap = tiny_skia::Pixmap::new(self.width, self.height).unwrap();
    }

    fn current_clip(&self) -> Option<&ClipEntry> {
        self.clip_stack.last()
    }
}

impl<D: raw_window_handle::HasDisplayHandle, W: raw_window_handle::HasWindowHandle> Backend
    for SkiaBackend<D, W>
{
    fn begin_frame(&mut self) {
        self.pixmap.fill(color_to_skia(&Colors::BLACK, 1.0));
        self.opacity_stack.clear();
        self.clip_stack.clear();
        self.transform_stack.clear();
        self.current_opacity = 1.0;
        self.current_transform = tiny_skia::Transform::identity();
    }

    fn execute(&mut self, commands: &[PaintCommand]) {
        for cmd in commands {
            match cmd {
                PaintCommand::FillRect {
                    rect,
                    radius,
                    paint,
                } => {
                    self.pixmap.fill_path(
                        &gen_rect_path(rect, radius),
                        &paint_to_skia(paint, self.current_blend, self.current_opacity, rect),
                        tiny_skia::FillRule::Winding,
                        self.current_transform,
                        None,
                    ) //TODO: Build a clip
                }
                PaintCommand::StrokeRect {
                    rect,
                    radius,
                    style,
                } => {
                    self.pixmap.stroke_path(
                        &gen_rect_path(rect, radius),
                        &paint_to_skia(&style.paint, self.current_blend, self.current_opacity, rect),
                        &gen_border_stroke(style.width),
                        self.current_transform,
                        None,
                    ) //TODO: Build a clip
                }
                PaintCommand::BoxShadow { rect, shadow } => {
                    todo!()
                }
                PaintCommand::DrawText { command } => {
                    todo!()
                }
                PaintCommand::DrawImage { rect, image, tint } => {
                    todo!()
                }
                PaintCommand::PushClip { rect, radius } => {
                    self.clip_stack.push(ClipEntry {
                        rect: *rect,
                        radius: *radius,
                    });
                }
                PaintCommand::PopClip => {
                    self.clip_stack.pop();
                }
                PaintCommand::PushOpacity { opacity } => {
                    self.opacity_stack.push(self.current_opacity);
                    self.current_opacity = *opacity;
                }
                PaintCommand::PopOpacity => {
                    if let Some(prev) = self.opacity_stack.pop() {
                        self.current_opacity = prev;
                    }
                }
                PaintCommand::PushBlend { mode } => {
                    self.blend_stack.push(self.current_blend);
                    self.current_blend = blend_to_skia(mode);
                }
                PaintCommand::PopBlend => {
                    if let Some(prev) = self.blend_stack.pop() {
                        self.current_blend = prev;
                    }
                }
                PaintCommand::PushTransform { transform } => {
                    self.transform_stack.push(self.current_transform);
                    self.current_transform = self
                        .current_transform
                        .pre_concat(transform_to_skia(transform));
                }
                PaintCommand::PopTransform => {
                    if let Some(prev) = self.transform_stack.pop() {
                        self.current_transform = prev;
                    }
                }
                PaintCommand::PushFilter { filter } => {
                    todo!()
                }
                PaintCommand::PopFilter => {
                    todo!()
                }
                PaintCommand::PushLayer => {
                    todo!()
                }
                PaintCommand::PopLayer => {
                    todo!()
                }
            }
        }
    }

    fn end_frame(&mut self) {
        let pixels = self.pixmap.pixels();
        let mut buffer = self.surface.buffer_mut().unwrap();

        for (dst, src) in buffer.iter_mut().zip(pixels.iter()) {
            let (r, g, b) = (src.red(), src.green(), src.blue());
            *dst = ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0);
        }

        buffer.present().unwrap();
    }
}

fn gen_rect_path(rect: &Rect, radius: &Corners<f32>) -> tiny_skia::Path {
    let x = rect.origin.x;
    let y = rect.origin.y;
    let w = rect.size.horizontal;
    let h = rect.size.vertical;

    //TODO: Verify Vibe Code
    if radius.top_left == 0.0
        && radius.top_right == 0.0
        && radius.bottom_left == 0.0
        && radius.bottom_right == 0.0
    {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        pb.finish().unwrap()
    } else {
        let tl = radius.top_left.min(w / 2.0).min(h / 2.0);
        let tr = radius.top_right.min(w / 2.0).min(h / 2.0);
        let br = radius.bottom_right.min(w / 2.0).min(h / 2.0);
        let bl = radius.bottom_left.min(w / 2.0).min(h / 2.0);

        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(x + tl, y);
        pb.line_to(x + w - tr, y);
        pb.quad_to(x + w, y,         x + w, y + tr);
        pb.line_to(x + w, y + h - br);
        pb.quad_to(x + w, y + h,     x + w - br, y + h);
        pb.line_to(x + bl, y + h);
        pb.quad_to(x,      y + h,    x, y + h - bl);
        pb.line_to(x, y + tl);
        pb.quad_to(x,      y,        x + tl, y);
        pb.close();
        pb.finish().unwrap()
    }
}

fn color_to_skia(c: &Color, opacity: f32) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(c.r, c.g, c.b, c.a * opacity).unwrap_or(tiny_skia::Color::BLACK)
}

fn stops_to_skia(stops: &ColorStops, opacity: f32) -> Vec<tiny_skia::GradientStop> {
    let mut result = vec![
        tiny_skia::GradientStop::new(stops.start.position, color_to_skia(&stops.start.color, opacity)),
    ];
    for s in &stops.mid {
        result.push(tiny_skia::GradientStop::new(s.position, color_to_skia(&s.color, opacity)));
    }
    result.push(tiny_skia::GradientStop::new(stops.end.position, color_to_skia(&stops.end.color, opacity)));
    result
}

fn paint_to_skia<'a>(paint: &Paint, blend: tiny_skia::BlendMode, opacity: f32, bounds: &Rect) -> tiny_skia::Paint<'a> {
    //TODO: Verify Vibe Code
    let mut sk = tiny_skia::Paint::default();
    sk.anti_alias = true;
    sk.blend_mode = blend;

    match paint {
        Paint::Color(c) => {
            sk.set_color(color_to_skia(c, opacity));
        }

        Paint::LinearGradient(g) => {
            let stops = stops_to_skia(&g.stops, opacity);
            // angle: 0 = top→bottom, 90 = left→right
            let angle_rad = g.angle.to_radians();
            let (sin, cos) = angle_rad.sin_cos();
            let cx = bounds.origin.x + bounds.size.horizontal  * 0.5;
            let cy = bounds.origin.y + bounds.size.vertical * 0.5;
            let half_w = bounds.size.horizontal  * 0.5;
            let half_h = bounds.size.vertical * 0.5;
            let start = tiny_skia::Point::from_xy(cx - sin * half_w, cy - cos * half_h);
            let end   = tiny_skia::Point::from_xy(cx + sin * half_w, cy + cos * half_h);

            if let Some(shader) = tiny_skia::LinearGradient::new(
                start, end, stops,
                tiny_skia::SpreadMode::Pad,
                tiny_skia::Transform::identity(),
            ) {
                sk.shader = shader;
            }
        }

        Paint::RadialGradient(g) => {
            let stops = stops_to_skia(&g.stops, opacity);
            let center = tiny_skia::Point::from_xy(
                bounds.origin.x + g.center.x * bounds.size.horizontal,
                bounds.origin.y + g.center.y * bounds.size.vertical,
            );
            let radius = bounds.size.horizontal.max(bounds.size.vertical) * 0.5;

            if let Some(shader) = tiny_skia::RadialGradient::new(
                center,                          // start_point (inner)
                0.0,                             // start_radius (point at center)
                center,                          // end_point (same center)
                radius,                          // end_radius (fills bounds)
                stops,
                tiny_skia::SpreadMode::Pad,
                tiny_skia::Transform::identity(),
            ) {
                sk.shader = shader;
            }
        }

        // tiny-skia has no conic or mesh gradient — magenta placeholder
        Paint::ConicGradient(_) | Paint::MeshGradient(_) => {
            sk.set_color(tiny_skia::Color::from_rgba8(255, 0, 255, 255));
        }
    }

    sk
}

fn transform_to_skia(transform: &Transform) -> tiny_skia::Transform {
    let m = transform.matrix;

    tiny_skia::Transform::from_row(m[0], m[1], m[3], m[4], m[6], m[7])
}

fn gen_border_stroke(width: f32) -> tiny_skia::Stroke {
    tiny_skia::Stroke {
        width,
        ..Default::default()
    }
}

fn blend_to_skia(blend: &BlendMode) -> tiny_skia::BlendMode {
    match blend {
        BlendMode::Normal      => tiny_skia::BlendMode::SourceOver,
        BlendMode::Multiply    => tiny_skia::BlendMode::Multiply,
        BlendMode::Screen      => tiny_skia::BlendMode::Screen,
        BlendMode::Overlay     => tiny_skia::BlendMode::Overlay,
        BlendMode::Darken      => tiny_skia::BlendMode::Darken,
        BlendMode::Lighten     => tiny_skia::BlendMode::Lighten,
        BlendMode::ColorDodge  => tiny_skia::BlendMode::ColorDodge,
        BlendMode::ColorBurn   => tiny_skia::BlendMode::ColorBurn,
        BlendMode::HardLight   => tiny_skia::BlendMode::HardLight,
        BlendMode::SoftLight   => tiny_skia::BlendMode::SoftLight,
        BlendMode::Difference  => tiny_skia::BlendMode::Difference,
        BlendMode::Exclusion   => tiny_skia::BlendMode::Exclusion,
        BlendMode::Hue         => tiny_skia::BlendMode::Hue,
        BlendMode::Saturation  => tiny_skia::BlendMode::Saturation,
        BlendMode::Color       => tiny_skia::BlendMode::Color,
        BlendMode::Luminosity  => tiny_skia::BlendMode::Luminosity,
    }
}