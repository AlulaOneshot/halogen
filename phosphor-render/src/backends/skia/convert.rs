//! Conversions from Phosphor style types to Skia equivalents.
//!
//! Not to be exposed to external code, this is purely internal.

use phosphor_core::color::Color;
use phosphor_core::style::{Corners, Rect, Texture};

/// Convert a phosphor color to a skia color4f
pub(crate) fn color_to_sk(c: Color) -> skia_safe::Color4f {
    skia_safe::Color4f::new(c.r, c.g, c.b, c.a)
}

/// Convert a phosphor rect to a skia rect
pub(crate) fn rect_to_sk(r: Rect) -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(r.x, r.y, r.width, r.height)
}

/// Convert a Phosphor `Rect` + `Corners` to a Skia `RRect`.
///
/// Radii are in CSS order: top-left, top-right, bottom-right, bottom-left.
/// Skia's `RRect::new_rect_radii` uses the same order.
pub(crate) fn rrect_to_sk(rect: Rect, radii: Corners<f32>) -> skia_safe::RRect {
    let sk_rect = rect_to_sk(rect);
    // Skia radii order: top-left, top-right, bottom-right, bottom-left
    let sk_radii = [
        skia_safe::Vector::new(radii.top_left, radii.top_left),
        skia_safe::Vector::new(radii.top_right, radii.top_right),
        skia_safe::Vector::new(radii.bottom_right, radii.bottom_right),
        skia_safe::Vector::new(radii.bottom_left, radii.bottom_left),
    ];
    skia_safe::RRect::new_rect_radii(sk_rect, &sk_radii)
}

/// Flatten `GradientStops` into parallel color and position vecs, sorted by position.
pub(crate) fn collect_stops(stops: &phosphor_core::style::GradientStops) -> (Vec<skia_safe::Color4f>, Vec<f32>) {
    let mut all: Vec<&phosphor_core::style::ColorStop> = std::iter::once(&stops.start)
        .chain(stops.mid.iter())
        .chain(std::iter::once(&stops.end))
        .collect();
    // Sort defensively — callers should already have stops in order, but enforce it.
    all.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let colors = all.iter().map(|s| color_to_sk(s.color)).collect();
    let positions = all.iter().map(|s| s.position).collect();
    (colors, positions)
}

pub(crate) fn linear_gradient_points(rect: Rect, angle_deg: f32) -> (skia_safe::Point, skia_safe::Point) {
    let angle_rad = angle_deg.to_radians();
    // CSS gradient direction vector: (sin θ, -cos θ)
    let dx = angle_rad.sin();
    let dy = -angle_rad.cos();

    let cx = rect.x + rect.width / 2.0;
    let cy = rect.y + rect.height / 2.0;

    // Extent: project the half-diagonals onto the direction vector
    let extent = rect.width / 2.0 * dx.abs() + rect.height / 2.0 * dy.abs();

    (
        skia_safe::Point::new(cx - dx * extent, cy - dy * extent),
        skia_safe:: Point::new(cx + dx * extent, cy + dy * extent),
    )
}

pub(crate) fn texture_to_shader(texture: &Texture, rect: Rect) -> Option<skia_safe::Shader> {
    let linear_cs = skia_safe::ColorSpace::new_srgb_linear();

    let interpolation = skia_safe::gradient::Interpolation::default();

    match texture {
        Texture::Color(_) => None,

        Texture::LinearGradient { angle, stops } => {
            let (colors, positions) = collect_stops(stops);
            let (start, end) = linear_gradient_points(rect, *angle);
            let gradient = skia_safe::gradient::Gradient::new(
                skia_safe::gradient::Colors::new(&colors, Some(&positions), skia_safe::TileMode::Clamp, skia_safe::ColorSpace::new_srgb_linear()),
                interpolation,
            );
            skia_safe::gradient::shaders::linear_gradient((start, end), &gradient, None)
        }

        Texture::RadialGradient { center, stops } => {
            let (colors, positions) = collect_stops(stops);
            let cx = rect.x + center.0 * rect.width;
            let cy = rect.y + center.1 * rect.height;
            let radius = ((rect.width / 2.0).powi(2) + (rect.height / 2.0).powi(2)).sqrt();
            let gradient = skia_safe::gradient::Gradient::new(
                skia_safe::gradient::Colors::new(&colors, Some(&positions), skia_safe::TileMode::Clamp, skia_safe::ColorSpace::new_srgb_linear()),
                interpolation,
            );
            skia_safe::gradient::shaders::radial_gradient((skia_safe::Point::new(cx, cy), radius), &gradient, None)
        }

        Texture::ConicGradient { center, angle, stops } => {
            let (colors, positions) = collect_stops(stops);
            let cx = rect.x + center.0 * rect.width;
            let cy = rect.y + center.1 * rect.height;
            let gradient = skia_safe::gradient::Gradient::new(
                skia_safe::gradient::Colors::new(&colors, Some(&positions), skia_safe::TileMode::Clamp, skia_safe::ColorSpace::new_srgb_linear()),
                interpolation,
            );
            skia_safe::gradient::shaders::sweep_gradient(skia_safe::Point::new(cx, cy), (*angle, angle + 360.0), &gradient, None)
        }

        Texture::MeshGradient { .. } => {
            // TODO: mesh gradients — no direct Skia primitive; requires manual tessellation
            // into a mesh of bilinear patches.
            None
        }
    }
}

pub(crate) fn texture_to_paint(texture: &Texture, rect: Rect, stroke_width: Option<f32>) -> skia_safe::Paint {
    let linear_cs = skia_safe::ColorSpace::new_srgb_linear();
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);

    match stroke_width {
        Some(w) => {
            paint.set_style(skia_safe::paint::Style::Stroke);
            paint.set_stroke_width(w);
        }
        None => {
            paint.set_style(skia_safe::paint::Style::Fill);
        }
    }

    match texture {
        Texture::Color(c) => {
            paint.set_color4f(color_to_sk(*c), &linear_cs);
        }
        _ => {
            if let Some(shader) = texture_to_shader(texture, rect) {
                paint.set_shader(shader);
            } else {
                // MeshGradient fallback: transparent
                paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.0), &linear_cs);
            }
        }
    }

    paint
}