use crate::color::Color;
use crate::geometry::{Corners, Rect};
use crate::style::{BorderStyles, BoxShadow, FontWeight, Paint, TextAlignment};
use crate::theme::Theme;
use crate::widget::PaintSink;
use std::sync::Arc;

pub enum PaintCommand {
    // basic rectangles
    FillRect {
        rect: Rect,
        radius: Corners<f32>,
        paint: Paint,
    },
    StrokeRect {
        rect: Rect,
        radius: Corners<f32>,
        style: BorderStyles,
    },
    BoxShadow {
        rect: Rect,
        shadow: BoxShadow,
    },

    //TODO: Paths

    DrawText {
        command: TextCommand
    },

    DrawImage {
        rect: Rect,
        image: ImageRef,
        tint: Option<Color>
    },

    // layers in push/pop pairs
    PushClip     { rect: Rect, radius: Corners<f32> },
    PopClip,
    PushOpacity  { opacity: f32 },
    PopOpacity,
    PushBlend    { mode: BlendMode },
    PopBlend,
    PushTransform { transform: Transform },
    PopTransform,
    PushFilter   { filter: Filter },
    PopFilter,

    // Z-ordering
    PushLayer,   // begin painting above normal flow
    PopLayer,
}

pub struct TextCommand {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub color: Color,
    pub bounds: Rect,
    pub align: TextAlignment,
    pub line_height: f32,
}

// Temporary system until we have concrete image support
pub struct ImageRef(pub Arc<dyn ImageData>);

pub trait ImageData: Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub enum BlendMode {
    Normal, Multiply, Screen, Overlay,
    Darken, Lighten, ColorDodge, ColorBurn,
    HardLight, SoftLight, Difference, Exclusion,
    Hue, Saturation, Color, Luminosity,
}

pub struct Transform {
    // Column-major 3x3 affine matrix
    pub matrix: [f32; 9],
}

impl Transform {
    pub fn translate(x: f32, y: f32) -> Self { todo!() }
    pub fn rotate(angle_rad: f32) -> Self { todo!() }
    pub fn scale(x: f32, y: f32) -> Self { todo!() }
}

pub enum Filter {
    Blur { sigma_x: f32, sigma_y: f32 },
    Backdrop { inner: Box<Filter> },  // blur behind the widget
    Brightness(f32),
    Contrast(f32),
    Saturate(f32),
    Grayscale(f32),
    Composed(Vec<Filter>),
}

pub struct PaintContext<'a> {
    pub bounds: Rect,
    pub theme: &'a Theme,
    pub(crate) renderer: &'a mut dyn PaintSink,
}

impl<'a> PaintContext<'a> {
    pub fn fill_rect(&mut self, rect: Rect, radius: Corners<f32>, paint: Paint) {
        self.renderer.push(PaintCommand::FillRect { rect, radius, paint });
    }

    pub fn stroke_rect(&mut self, rect: Rect, radius: Corners<f32>, style: BorderStyles) {
        self.renderer.push(PaintCommand::StrokeRect { rect, radius, style });
    }

    pub fn box_shadow(&mut self, rect: Rect, shadow: crate::style::BoxShadow) {
        self.renderer.push(PaintCommand::BoxShadow { rect, shadow });
    }

    pub fn draw_text(&mut self, cmd: TextCommand) {
        self.renderer.push(PaintCommand::DrawText { command: cmd });
    }

    pub fn draw_image(&mut self, rect: Rect, image: ImageRef, tint: Option<crate::color::Color>) {
        self.renderer.push(PaintCommand::DrawImage { rect, image, tint });
    }

    // ── Layer helpers ─────────────────────────────────────────────────────────

    pub fn with_clip(&mut self, rect: Rect, radius: Corners<f32>, f: impl FnOnce(&mut PaintContext)) {
        self.renderer.push(PaintCommand::PushClip { rect, radius });
        f(self);
        self.renderer.push(PaintCommand::PopClip);
    }

    pub fn with_opacity(&mut self, opacity: f32, f: impl FnOnce(&mut PaintContext)) {
        self.renderer.push(PaintCommand::PushOpacity { opacity });
        f(self);
        self.renderer.push(PaintCommand::PopOpacity);
    }

    pub fn with_blend(&mut self, mode: BlendMode, f: impl FnOnce(&mut PaintContext)) {
        self.renderer.push(PaintCommand::PushBlend { mode });
        f(self);
        self.renderer.push(PaintCommand::PopBlend);
    }

    pub fn with_transform(&mut self, transform: Transform, f: impl FnOnce(&mut PaintContext)) {
        self.renderer.push(PaintCommand::PushTransform { transform });
        f(self);
        self.renderer.push(PaintCommand::PopTransform);
    }

    pub fn with_filter(&mut self, filter: Filter, f: impl FnOnce(&mut PaintContext)) {
        self.renderer.push(PaintCommand::PushFilter { filter });
        f(self);
        self.renderer.push(PaintCommand::PopFilter);
    }
}