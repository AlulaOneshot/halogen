use crate::geometry::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn linear_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn linear_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: 1.0,
        }
    }

    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: if a == 255 {
                1.0
            } else if a == 0 {
                0.0
            } else {
                a as f32 / 255.0
            },
        }
    }

    pub fn hex(_s: impl Into<String>) -> Self {
        todo!()
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }
}

impl Default for Color {
    fn default() -> Self {
        Colors::TRANSPARENT
    }
}

#[allow(clippy::excessive_precision)]
fn srgb_to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb * 0.0773993808
    } else {
        // Gamma segment, do this weird math
        f32::powf(srgb * 0.9478672986 + 0.0521327014, 2.4)
    }
}

pub struct Colors;

impl Colors {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorStop {
    ///NOTE: At usage, this must be clamped to `[0.0, 1.0]` to prevent bugs, as there is no way to clamp it within the struct.
    pub position: f32,
    pub color: Color,
}

impl ColorStop {
    pub fn new(position: f32, color: Color) -> Self {
        Self { position: position.clamp(0.0, 1.0), color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStops {
    pub start: ColorStop,
    pub end: ColorStop,
    pub mid: Vec<ColorStop>,
}

impl ColorStops {
    pub fn new(start: ColorStop, end: ColorStop) -> Self {
        Self {
            start,
            end,
            mid: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub angle: f32,
    pub stops: ColorStops
}

impl LinearGradient {
    pub fn new(angle: f32, stops: ColorStops) -> Self {
        Self { angle, stops }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    ///NOTE: At usage, this must be clamped to `[0.0, 1.0]` to prevent bugs, as there is no way to clamp it within the struct.
    pub center: Point<f32>,
    pub stops: ColorStops
}

impl RadialGradient {
    pub fn new(center: Point<f32>, stops: ColorStops) -> Self {
        Self { center, stops }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    ///NOTE: At usage, this must be clamped to `[0.0, 1.0]` to prevent bugs, as there is no way to clamp it within the struct.
    pub center: Point<f32>,
    pub angle: f32,
    pub stops: ColorStops
}

impl ConicGradient {
    pub fn new(center: Point<f32>, angle: f32, stops: ColorStops) -> Self {
        Self { center, angle, stops }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshPoint {
    pub position: Point<f32>,
    pub color: Color,
}

impl MeshPoint {
    pub fn new(position: Point<f32>, color: Color) -> Self {
        Self { position, color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshGradient {
    pub columns: usize,
    pub rows: usize,
    pub points: Vec<MeshPoint>,
}

impl MeshGradient {
    pub fn new(columns: usize, rows: usize, points: Vec<MeshPoint>) -> Self {
        assert_eq!(columns * rows, points.len(), "columns * rows must always equal points.len(). columns * rows: {}, points.len(): {}", columns * rows, points.len());
        Self { columns, rows, points }
    }
}