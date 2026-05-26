//! Color types and constructors.
//!
//! [`Color`] stores all values as **linear-light RGBA** (`f32` per channel).
//! Every constructor that accepts standard sRGB converts it to linear RGBA.
//! Values above `1.0` are correct and intentionally preserved (except in the case of alpha). Values below `0.0` are always invalid.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red channel, linear light. Values above `1.0` are valid for HDR.
    pub r: f32,
    /// Green channel, linear light. Values above `1.0` are valid for HDR.
    pub g: f32,
    /// Blue channel, linear light. Values above `1.0` are valid for HDR.
    pub b: f32,
    /// Alpha channel. Clamped to `[0.0, 1.0]` on construction.
    pub a: f32,
}

impl Color {
    /// Construct from raw linear RGBA.
    /// Skips any gamma transfer. If you want to use normal rgb, use the rgb8/rgba8 function.
    /// Alpha is clamped to `[0.0, 1.0]`, RGB above `1.0` is valid for HDR.
    pub const fn linear_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a: a.clamp(0.0, 1.0) }
    }

    /// Construct from sRGB `u8` components `[0, 255]`.
    ///
    /// Applies the sRGB -> linear transfer function. Alpha is 1.0.
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: 1.0,
        }
    }

    /// Construct from sRGB `u8` components `[0, 255]`.
    ///
    /// Applies the sRGB -> linear transfer function. Alpha is `[0, 255] / 255.0`.
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: a as f32 / 255.0,
        }
    }

    /// Construct from a CSS hex string of 6 characters (`RRGGBB`) or 8 characters (`RRGGBBAA`).
    ///
    /// The leading hash is optional, input is treated as sRGB and decoded to linear.
    ///
    /// On malformed input, a panic occurs.
    pub fn hex(s: impl Into<String>) -> Self {
        let s = s.into();
        let s = s.trim_start_matches('#');
        let p = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).expect("Malformed input for hex color.") as f32 / 255.0;
        match s.len() {
            6 => Self::srgb(p(0), p(2), p(4)),
            8 => Self::srgb_alpha(p(0), p(2), p(4), p(6)),
            _ => Self::rgb8(0, 0, 0),
        }
    }

    /// Construct from sRGB `[0.0, 1.0]` components. Alpha is 1.0.
    ///
    /// Applies the sRGB -> linear transfer function on RGB.
    pub fn srgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: srgb_to_linear(r),
            g: srgb_to_linear(g),
            b: srgb_to_linear(b),
            a: 1.0,
        }
    }

    /// Construct from sRGB `[0.0, 1.0]` with explicit alpha.
    ///
    /// Applies the sRGB -> linear transfer function on RGB. Alpha is clamped to `[0.0, 1.0]`.
    pub fn srgb_alpha(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { a: a.clamp(0.0, 1.0), ..Self::srgb(r, g, b) }
    }

    /// Construct from HSL + alpha.
    ///
    /// - `h`: hue in degrees, wraps into `[0, 360)`
    /// - `s`: saturation `[0.0, 1.0]`
    /// - `l`: lightness `[0.0, 1.0]`
    /// - `a`: alpha `[0.0, 1.0]`, clamped
    ///
    /// Uses the W3C HSL -> sRGB algorithm, then decodes to linear.
    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        // Convert HSL → sRGB → linear
        let (r, g, b) = hsl_to_srgb(h, s, l);
        Self::srgb_alpha(r, g, b, a.clamp(0.0, 1.0))
    }

    /// Construct from HSL. Alpha is 1.0. See [`Color::hsla`] for parameter ranges.
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::hsla(h, s, l, 1.0)
    }

    //TODO: OKLab and OKLCH

    /// Return a copy of this color with alpha replaced. Alpha is clamped to `[0.0, 1.0]`.
    pub const fn with_alpha(mut self, a: f32) -> Self {
        self.a = a.clamp(0.0, 1.0);
        self
    }

    //TODO: Darken and lighten VIA OKLab Space

    //TODO: Mix Colors VIA OKLab Space
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Color::hex(s)
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Color::hex(&s)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self::linear_rgba(r, g, b, a)
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self::linear_rgba(r, g, b, 1.0)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self::rgba8(r, g, b, a)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::rgb8(r, g, b)
    }
}

/// Common color constants in linear-light RGBA.
pub struct Colors;

impl Colors {
    /// Opaque black - linear `(0, 0, 0, 1)`.
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    /// Opaque white - linear `(1, 1, 1, 1)`.
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    /// Fully transparent black - linear `(0, 0, 0, 0)`.
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}

#[allow(clippy::excessive_precision)]
fn srgb_to_linear(srgb: f32) -> f32 {
    // Linear Segment: close enough to a linear that just scaling is ok.
    if srgb <= 0.04045 {
        srgb * 0.0773993808
    } else {
        // Gamma segment, do this weird math
        (srgb * 0.9478672986 + 0.0521327014).powf(2.4)
    }
}

/// W3C single-function HSL -> sRGB algorithm.
fn hsl_to_srgb(mut hue: f32, sat: f32, light: f32) -> (f32, f32, f32) {
    // Normalize to [0.0, 359.99999999].
    hue %= 360.0;
    // In the case that hue was 360, add 360 back.
    if hue < 0.0 { hue += 360.0; }
    // Normalize to [0.0, 1.0]
    hue /= 360.0;

    // Computes one channel using piecewise linear approximation of hue-to-RGB mapping.
    let f = |n: f32| {
        // Rotate the starting offset by channel.
        let k = (n + hue * 12.0) % 12.0;
        // Chroma
        let a = sat * light.min(1.0 - light);
        light - a * (-1.0_f32).max((k - 3.0).min((9.0 - k).min(1.0)))
    };

    (f(0.0), f(8.0), f(4.0))
}