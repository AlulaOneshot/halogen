//! Layout, paint, style, and text types.
//! The primary public surface is the [`Style`] trait and its standard implementation [`WidgetStyle`].
//! The trait is object-safe and allows animated or theme-driven implementations without coupling to [`WidgetStyle`]'s fields.
//!
//! ## Units
//!
//! Phosphor uses three length units:
//!
//! - **`Px(f32)`** - logical pixels, device-independent.
//! - **`Percent(f32)`** - relative to the parent container. Pass `50.0` for 50%.
//! - **[`ScaleUnits`]** - 1:1 with logical pixels at 1280×720. Scales linearly with the viewport. Use for all fixed-reference sizes so the UI is consistent across resolutions without manual math.
//!
//! ## Layout model
//!
//! Phosphor uses Flexbox via taffy, with plans to impliment a grid layout. `box-sizing` is *always* `border-box`.
//! The root container takes up the full viewport. All layout types ultimately convert to [`taffy::Style`] via [`Style::to_taffy`].
//!
//! ## Paint types
//!
//! [`Texture`] is the fundamental paint value - a solid color, or one of four gradient types, as well as images eventually. It's used for both backgrounds (`Background`) and text color (`TextStyles::text_paint`), so gradient text is first-class.

use crate::color::{Color, Colors};

/// A unit that scales proportionally with the viewport, 1:1 at 1280×720.
///
/// Use for any size that should feel consistent across resolutions.
/// At 2560×1440 (2× 1280×720), a `ScaleUnits(100.0)` value resolves to 200px.
///
/// **Cannot be used for `flex_basis`** - see [`Dimension::to_flex_basis`].
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct ScaleUnits(pub f32);

// ===================
// Measurement structs
// ===================

/// The logical pixel size of the rendering surface.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
}

impl ViewportSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// A single dimension value used for width, height, padding, margin, gaps, and insets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// Taffy determines the value. Meaning depends on context (shrink-to-fit, fill parent, etc.). Default.
    Auto,
    /// Logical pixels.
    Px(f32),
    /// Percentage of the parent's size on the relevant axis. Pass `50.0` for 50%.
    Percent(f32),
    /// Scales with the viewport; 1:1 at 1280×720. See [`ScaleUnits`].
    ScaleUnits(ScaleUnits),
}

impl Dimension {
    pub fn px(v: f32) -> Self {
        Self::Px(v)
    }
    pub fn percent(v: f32) -> Self {
        Self::Percent(v)
    }
    /// 100% of the parent on this axis.
    pub fn full() -> Self {
        Self::Percent(100.0)
    }
    /// 50% of the parent on this axis.
    pub fn half() -> Self {
        Self::Percent(50.0)
    }
    pub fn auto() -> Self {
        Self::Auto
    }

    fn to_taffy_x(self, scale_x: f32) -> taffy::Dimension {
        match self {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Px(v) => taffy::Dimension::length(v),
            Dimension::Percent(v) => taffy::Dimension::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::Dimension::length(s.0 * scale_x),
        }
    }

    fn to_taffy_y(self, scale_y: f32) -> taffy::Dimension {
        match self {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Px(v) => taffy::Dimension::length(v),
            Dimension::Percent(v) => taffy::Dimension::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::Dimension::length(s.0 * scale_y),
        }
    }

    fn to_lpa_x(self, scale_x: f32) -> taffy::LengthPercentageAuto {
        match self {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Px(v) => taffy::LengthPercentageAuto::length(v),
            Dimension::Percent(v) => taffy::LengthPercentageAuto::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::LengthPercentageAuto::length(s.0 * scale_x),
        }
    }

    fn to_lpa_y(self, scale_y: f32) -> taffy::LengthPercentageAuto {
        match self {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Px(v) => taffy::LengthPercentageAuto::length(v),
            Dimension::Percent(v) => taffy::LengthPercentageAuto::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::LengthPercentageAuto::length(s.0 * scale_y),
        }
    }

    pub fn to_lp_x(self, scale_x: f32) -> taffy::LengthPercentage {
        match self {
            Dimension::Auto => taffy::LengthPercentage::length(0.0),
            Dimension::Px(v) => taffy::LengthPercentage::length(v),
            Dimension::Percent(v) => taffy::LengthPercentage::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::LengthPercentage::length(s.0 * scale_x),
        }
    }

    pub fn to_lp_y(self, scale_y: f32) -> taffy::LengthPercentage {
        match self {
            Dimension::Auto => taffy::LengthPercentage::length(0.0),
            Dimension::Px(v) => taffy::LengthPercentage::length(v),
            Dimension::Percent(v) => taffy::LengthPercentage::percent(v / 100.0),
            Dimension::ScaleUnits(s) => taffy::LengthPercentage::length(s.0 * scale_y),
        }
    }

    /// Convert to a Taffy `flex_basis` dimension.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `ScaleUnits`. Flex basis is axis-dependent and
    /// `ScaleUnits` resolution requires knowing the axis at layout time -
    /// a limitation not yet resolved. Use `Px` or `Percent` for `flex_basis`.
    pub fn to_flex_basis(self) -> taffy::Dimension {
        match self {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Px(px) => taffy::Dimension::length(px),
            Dimension::Percent(p) => taffy::Dimension::percent(p / 100.0),
            Dimension::ScaleUnits(_) => {
                panic!("ScaleUnits is not supported for flex_basis - use Px or Percent instead")
            }
        }
    }
}

impl From<f32> for Dimension {
    /// Treats the value as logical pixels.
    fn from(v: f32) -> Self {
        Self::Px(v)
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Auto
    }
}

/// Per-edge values for padding, margin, border, and insets.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Edges<T: Clone> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone> Edges<T> {
    /// Same value on all four edges.
    pub fn all(v: T) -> Self {
        Self {
            top: v.clone(),
            right: v.clone(),
            bottom: v.clone(),
            left: v,
        }
    }

    /// `x` on left/right, `y` on top/bottom.
    pub fn xy(x: T, y: T) -> Self {
        Self {
            top: y.clone(),
            right: x.clone(),
            bottom: y,
            left: x,
        }
    }

    /// Each edge individually, in CSS order (top, right, bottom, left).
    pub fn sides(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn with_top(mut self, v: T) -> Self {
        self.top = v;
        self
    }

    pub fn with_bottom(mut self, v: T) -> Self {
        self.bottom = v;
        self
    }

    pub fn with_left(mut self, v: T) -> Self {
        self.left = v;
        self
    }

    pub fn with_right(mut self, v: T) -> Self {
        self.right = v;
        self
    }
}

impl Edges<Dimension> {
    fn to_taffy_margin(
        &self,
        scale_x: f32,
        scale_y: f32,
    ) -> taffy::Rect<taffy::LengthPercentageAuto> {
        taffy::Rect {
            top: self.top.to_lpa_y(scale_y),
            bottom: self.bottom.to_lpa_y(scale_y),
            left: self.left.to_lpa_x(scale_x),
            right: self.right.to_lpa_x(scale_x),
        }
    }

    fn to_taffy_padding(
        &self,
        scale_x: f32,
        scale_y: f32,
    ) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            top: self.top.to_lp_y(scale_y),
            bottom: self.bottom.to_lp_y(scale_y),
            left: self.left.to_lp_x(scale_x),
            right: self.right.to_lp_x(scale_x),
        }
    }
}

impl<T: Clone> From<T> for Edges<T> {
    fn from(value: T) -> Self {
        Self::all(value)
    }
}

/// Per-corner values for border radii.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Corners<T: Clone> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

impl<T: Clone> Corners<T> {
    /// Same value on all four corners.
    pub fn all(v: T) -> Self {
        Self {
            top_left: v.clone(),
            top_right: v.clone(),
            bottom_right: v.clone(),
            bottom_left: v,
        }
    }

    /// Each corner individually, to be noted that this is not CSS order.
    pub fn individual(top_left: T, top_right: T, bottom_left: T, bottom_right: T) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn with_top(mut self, radius: T) -> Self {
        self.top_left = radius.clone();
        self.top_right = radius;
        self
    }

    pub fn with_bottom(mut self, radius: T) -> Self {
        self.bottom_left = radius.clone();
        self.bottom_right = radius;
        self
    }

    pub fn with_left(mut self, radius: T) -> Self {
        self.top_left = radius.clone();
        self.bottom_left = radius;
        self
    }

    pub fn with_right(mut self, radius: T) -> Self {
        self.top_right = radius.clone();
        self.bottom_right = radius;
        self
    }
}

impl<T: Clone + Default> Corners<T> {
    pub fn from_top(v: T) -> Self {
        Self {
            top_left: v.clone(),
            top_right: v,
            ..Default::default()
        }
    }

    pub fn from_bottom(mut self, v: T) -> Self {
        Self {
            bottom_left: v.clone(),
            bottom_right: v,
            ..Default::default()
        }
    }

    pub fn from_left(mut self, v: T) -> Self {
        Self {
            top_left: v.clone(),
            bottom_left: v,
            ..Default::default()
        }
    }

    pub fn from_right(mut self, v: T) -> Self {
        Self {
            top_right: v.clone(),
            bottom_right: v,
            ..Default::default()
        }
    }
}

impl<T: Clone> From<T> for Corners<T> {
    fn from(value: T) -> Self {
        Self::all(value)
    }
}

/// How a widget's size is determined on one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
    /// Fixed to exactly this dimension.
    Exact(Dimension),
    /// Constrained between optional min and max.
    ///
    /// `None` for a bound means unconstrained on that end:
    /// `MinMax(None, None)` is fully unconstrained (Taffy decides), which is the default.
    MinMax(Option<Dimension>, Option<Dimension>),
}

impl From<Dimension> for Sizing {
    fn from(value: Dimension) -> Self {
        Self::Exact(value)
    }
}

impl From<(Option<Dimension>, Option<Dimension>)> for Sizing {
    fn from((min, max): (Option<Dimension>, Option<Dimension>)) -> Self {
        Self::MinMax(min, max)
    }
}

impl Sizing {
    /// Returns `(size, min_size, max_size)` in Taffy's Dimension type for the X axis.
    fn to_taffy_x(self, scale: f32) -> (taffy::Dimension, taffy::Dimension, taffy::Dimension) {
        match self {
            Sizing::Exact(d) => (
                d.to_taffy_x(scale),
                taffy::Dimension::auto(),
                taffy::Dimension::auto(),
            ),
            Sizing::MinMax(min, max) => (
                taffy::Dimension::auto(),
                min.map(|d| d.to_taffy_x(scale))
                    .unwrap_or(taffy::Dimension::auto()),
                max.map(|d| d.to_taffy_x(scale))
                    .unwrap_or(taffy::Dimension::auto()),
            ),
        }
    }

    /// Returns `(size, min_size, max_size)` in Taffy's Dimension type for the Y axis.
    fn to_taffy_y(self, scale: f32) -> (taffy::Dimension, taffy::Dimension, taffy::Dimension) {
        match self {
            Sizing::Exact(d) => (
                d.to_taffy_y(scale),
                taffy::Dimension::auto(),
                taffy::Dimension::auto(),
            ),
            Sizing::MinMax(min, max) => (
                taffy::Dimension::auto(),
                min.map(|d| d.to_taffy_y(scale))
                    .unwrap_or(taffy::Dimension::auto()),
                max.map(|d| d.to_taffy_y(scale))
                    .unwrap_or(taffy::Dimension::auto()),
            ),
        }
    }
}

impl Default for Sizing {
    /// Defaults to fully unconstrained (`MinMax(None, None)`).
    fn default() -> Self {
        Sizing::MinMax(None, None)
    }
}

/// Font size, either in logical pixels or scale units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    Px(f32),
    /// Scales with viewport; 1:1 at 1280×720. See [`ScaleUnits`].
    ScaleUnits(f32),
}

/// Absolute screen position and size, resolved after layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// ========
// The Rest
// ========

/// How a widget participates in layout.
///
/// Flex properties (`align_items`, `justify_content`, `flex_direction`) are
/// embedded directly inside `Display::Flex`. This makes it a type error to set
/// flex container properties on `Display::None` - correctness is enforced at
/// compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    /// Standard flexbox container. Default.
    Flex {
        direction: FlexDirection,
        align_items: Align,
        justify_content: Justify,
    },
    //TODO: Grid
    /// Widget is not rendered and takes no space in the layout.
    None,
}

impl Default for Display {
    fn default() -> Self {
        Display::Flex {
            direction: FlexDirection::default(),
            align_items: Align::default(),
            justify_content: Justify::default(),
        }
    }
}

/// Main axis direction for a flex container.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    /// Children line up left to right.
    #[default]
    Row,
    /// Children stack top to bottom
    Column,
    /// Children line up right to left
    RowReverse,
    /// Children stack bottom to top
    ColumnReverse,
}

/// Cross-axis alignment of children within a flex container.
///
/// This is the `align-items` property - set on the **container**, not the child.
/// For per-child override, see [`FlexProps`] / [`WidgetStyle::with_align_self`].
///
/// Note: `SpaceBetween`, `SpaceAround`, and `SpaceEvenly` are not valid here -
/// those are main-axis properties that belong on [`Justify`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Align {
    /// Children stretch to fill the cross axis. Default.
    #[default]
    Stretch,
    /// Pack toward the start of the cross axis.
    Start,
    /// Center along the cross axis.
    Center,
    /// Pack toward the end of the cross axis.
    End,
    /// Align to text baseline.
    Baseline,
}

/// Main-axis distribution of children in a flex container.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Justify {
    /// Pack toward the start of the main axis. Default.
    #[default]
    Start,
    /// Center along the main axis.
    Center,
    /// Pack toward the end of the main axis.
    End,
    /// First child at start, last at end, equal gaps between the rest.
    SpaceBetween,
    /// Equal space around each child (half-sized at edges).
    SpaceAround,
    /// Equal space around each child including edges.
    SpaceEvenly,
}

/// How content that overflows this widget's bounds is handled.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Overflow {
    /// Children render outside bounds. Default.
    #[default]
    Visible,
    /// Content outside bounds is clipped.
    Hidden,
    /// Treated as `Hidden` for layout purposes.
    ///
    /// A semantic marker for widgets that implement scrolling themselves -
    /// the actual scroll offset is tracked separately.
    Scroll,
}

impl Overflow {
    fn to_taffy(&self) -> taffy::Overflow {
        match self {
            Overflow::Visible => taffy::Overflow::Visible,
            Overflow::Hidden => taffy::Overflow::Hidden,
            Overflow::Scroll => taffy::Overflow::Hidden, // scroll offset managed externally
        }
    }
}

/// How a widget is positioned relative to the layout flow.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Position {
    /// Participates in normal flex/grid flow. Default.
    #[default]
    Relative,
    /// Removed from the flex flow; positioned by `offset` relative to the
    /// nearest ancestor that is also `Absolute`.
    Absolute {
        /// Inset from each edge of the containing block.
        offset: Edges<Dimension>,
    },
}

/// A paint value: a solid color or one of four gradient types.
///
/// Used for both backgrounds ([`Background`]) and text color
/// ([`TextStyles::text_paint`]). Gradient text maps to a Skia shader at paint time.
#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Color(Color),
    LinearGradient {
        /// Angle in degrees. `0` = bottom-to-top, `90` = left-to-right (CSS convention).
        angle: f32,
        stops: GradientStops,
    },
    RadialGradient {
        /// Center point, normalized to `[0.0, 1.0]` within the widget's bounds.
        center: (f32, f32),
        stops: GradientStops,
    },
    ConicGradient {
        /// Center point, normalized to `[0.0, 1.0]`.
        center: (f32, f32),
        /// Starting angle in degrees.
        angle: f32,
        stops: GradientStops,
    },
    MeshGradient {
        columns: usize,
        rows: usize,
        /// Row major order (left->right, top->bottom)
        /// Must have exactly `columns * rows` entries.
        points: Vec<MeshPoint>,
    },
    //TODO: Image
}

impl Texture {
    pub fn color(color: Color) -> Self {
        Self::Color(color)
    }

    pub fn linear(angle: f32, stops: GradientStops) -> Self {
        Self::LinearGradient { angle, stops }
    }

    pub fn radial(center: (f32, f32), stops: GradientStops) -> Self {
        Self::RadialGradient { center, stops }
    }

    pub fn conic(center: (f32, f32), angle: f32, stops: GradientStops) -> Self {
        Self::ConicGradient {
            center,
            stops,
            angle,
        }
    }

    /// # Panics
    ///
    /// Panics if `points.len() != columns * rows`.
    pub fn mesh(columns: usize, rows: usize, points: Vec<MeshPoint>) -> Self {
        assert_eq!(
            points.len(),
            columns * rows,
            "MeshGradient requires exactly columns * rows points. Expected {} ({} * {}) but got {}. If you want to exclude a color in a specific cell, use a transparent color.",
            columns * rows,
            columns,
            rows,
            points.len()
        );
        Self::MeshGradient {
            columns,
            rows,
            points,
        }
    }
}

impl From<Color> for Texture {
    fn from(color: Color) -> Self {
        Self::color(color)
    }
}

/// A control point in a mesh gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshPoint {
    /// Position normalized to `[0.0, 1.0]` within the widget's bounds.
    pub position: (f32, f32),
    pub color: Color,
}

/// A single color stop in a gradient.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    /// Position along the gradient axis, `[0.0, 1.0]`.
    pub position: f32,
    pub color: Color,
}

impl ColorStop {
    pub fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

impl From<(f32, Color)> for ColorStop {
    fn from((position, color): (f32, Color)) -> Self {
        Self { position, color }
    }
}

impl From<(f32, String)> for ColorStop {
    fn from((position, color): (f32, String)) -> Self {
        Self { position, color: color.into() }
    }
}

impl From<(f32, &str)> for ColorStop {
    fn from((position, color): (f32, &str)) -> Self {
        Self { position, color: color.into() }
    }
}

/// The color stops for a gradient.
///
/// Enforces a minimum of two stops at the type level. A gradient without
/// a start and end is nonsensical. Additional stops go in `mid`.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientStops {
    pub start: ColorStop,
    pub end: ColorStop,
    /// Optional stops between start and end.
    pub mid: Vec<ColorStop>,
}

impl GradientStops {
    /// Two-stop gradient (start and end only).
    pub fn new(start: impl Into<ColorStop>, end: impl Into<ColorStop>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
            mid: vec![],
        }
    }

    /// Gradient with additional intermediate stops.
    pub fn with_stops(start: ColorStop, end: ColorStop, mid: Vec<ColorStop>) -> Self {
        Self { start, end, mid }
    }
}

/// Background paint for a widget. Wraps a [`Texture`].
#[derive(Debug, Clone, PartialEq)]
pub struct Background(pub Texture);

impl Background {
    pub fn color(color: Color) -> Self {
        Self(Texture::color(color))
    }

    pub fn linear(angle: f32, stops: GradientStops) -> Self {
        Self(Texture::linear(angle, stops))
    }

    pub fn radial(center: (f32, f32), stops: GradientStops) -> Self {
        Self(Texture::radial(center, stops))
    }

    pub fn conic(center: (f32, f32), angle: f32, stops: GradientStops) -> Self {
        Self(Texture::conic(center, angle, stops))
    }

    /// See [`Texture::mesh`] for the point-count invariant.
    pub fn mesh(columns: usize, rows: usize, points: Vec<MeshPoint>) -> Self {
        Self(Texture::mesh(columns, rows, points))
    }
}

impl From<Background> for Texture {
    fn from(value: Background) -> Self {
        value.0
    }
}

impl From<Texture> for Background {
    fn from(texture: Texture) -> Self {
        Self(texture)
    }
}

/// The appearance of a single border edge.
#[derive(Debug, Clone, PartialEq)]
pub struct BorderEdge {
    /// Width in logical pixels.
    pub width: f32,
    /// Paint for this edge. Supports gradients.
    pub texture: Texture,
}

impl BorderEdge {
    pub fn new(width: f32, texture: impl Into<Texture>) -> Self {
        Self { width, texture: texture.into() }
    }

    pub fn from_texture(texture: impl Into<Texture>) -> Self {
        Self::new(1.0, texture)
    }

    pub fn from_width(width: f32) -> Self {
        Self::new(width, Colors::BLACK)
    }
}

impl From<(f32, Texture)> for BorderEdge {
    fn from((width, texture): (f32, Texture)) -> Self {
        Self {
            width,
            texture
        }
    }
}

impl From<(f32, Color)> for BorderEdge {
    fn from((width, color): (f32, Color)) -> Self {
        Self {
            width,
            texture: color.into()
        }
    }
}

impl Default for BorderEdge {
    fn default() -> Self {
        Self::new(0.0, Colors::TRANSPARENT)
    }
}

/// Border width, paint, and corner radii.
#[derive(Debug, Clone, PartialEq)]
pub struct BorderStyle {
    pub edges: Edges<BorderEdge>,
    /// Corner radii in logical pixels.
    pub radius: Corners<f32>,
}

impl BorderStyle {
    pub fn new(edges: impl Into<Edges<BorderEdge>>, radius: impl Into<Corners<f32>>) -> Self {
        Self { edges: edges.into(), radius: radius.into() }
    }

    pub fn from_edges(edges: Edges<BorderEdge>, radius: Corners<f32>) -> Self {
        Self { edges, radius }
    }

    pub fn from_corners(corners: impl Into<Corners<f32>>) -> Self {
        let corners = corners.into();
        Self {
            edges: Default::default(),
            radius: Corners::default(),
        }
    }

    pub fn with_radius(mut self, radius: Corners<f32>) -> Self {
        self.radius = radius;
        self
    }

    fn to_taffy_border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            top: taffy::LengthPercentage::length(self.edges.top.width),
            bottom: taffy::LengthPercentage::length(self.edges.bottom.width),
            left: taffy::LengthPercentage::length(self.edges.left.width),
            right: taffy::LengthPercentage::length(self.edges.right.width),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShadowSizing {
    Pixels(f32),
    ScaleUnits(ScaleUnits)
}

/// A CSS-style box shadow.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    /// (x, y) pixel offset from the widget.
    pub offset: (ShadowSizing, ShadowSizing),
    /// Gaussian blur radius in pixels.
    pub blur: f32,
    /// Extra expansion of the shadow beyond the widget's bounds.
    pub spread: f32,
    pub color: Color,
    /// If `true`, the shadow is rendered inside the widget's bounds.
    pub inset: bool,
}

impl BoxShadow {
    /// Construct a standard drop shadow (no spread, not inset).
    pub fn drop(offset: (ShadowSizing, ShadowSizing), blur: f32, color: Color) -> Self {
        Self {
            offset,
            blur,
            spread: 0.0,
            color,
            inset: false,
        }
    }
}

/// CSS font weight levels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

/// Horizontal text alignment within the widget's bounds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
    /// Justify text to both edges (last line is start-aligned).
    Justify,
}

/// Text rendering properties.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyles {
    /// Paint for the text glyphs. Supports gradients for gradient text.
    pub text_paint: Texture,
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    /// Must be a registered font family name, or a panic will occur at paint time.
    pub font_family: &'static str,
    pub text_align: TextAlign,
    /// Line spacing multiplier. `1.0` = lines touching, `1.5` = default, `2.0` = double-spaced.
    pub line_height: f32,
}

/// Column and row gap between flex/grid children.
///
/// Stored as `Dimension` (not `Option<Dimension>`) - `Px(0.0)` is explicit "no gap".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GapStyles {
    pub column_gap: Dimension,
    pub row_gap: Dimension,
}

/// Custom text measurement callback for Taffy.
///
/// Required for text-bearing widgets so Taffy can call back for size during layout.
/// Parameters: `(known_size, available_space) -> measured_size`.
pub type MeasureFunc = Box<
    dyn Fn(
            // Known Size
            taffy::Size<Option<f32>>,
            // Available space
            taffy::Size<taffy::AvailableSpace>,
        ) -> taffy::Size<f32>
        + Send
        + Sync,
>;

/// Overflow behavior on each axis independently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverflowAxes {
    pub x: Overflow,
    pub y: Overflow,
}

/// Flex child properties. All fields are optional; `None` defers to Taffy's defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexProps {
    /// How much the child grows to fill available space. Taffy default: `0.0`.
    pub flex_grow: Option<f32>,
    /// How much the child shrinks when space is tight. Taffy default: `1.0`.
    pub flex_shrink: Option<f32>,
    /// Initial main-axis size before grow/shrink. Cannot be `ScaleUnits`.
    pub flex_basis: Option<Dimension>,
}

/// Background, border, shadows, and opacity for a widget.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintStyles {
    pub background: Background,
    pub border: BorderStyle,
    pub shadows: Vec<BoxShadow>,
    /// Overall widget opacity, `[0.0, 1.0]`.
    pub opacity: f32,
}

/// The width and height sizing for a widget
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SizingAxes {
    pub width: Sizing,
    pub height: Sizing,
}

impl SizingAxes {
    /// Width and height sizing
    pub fn new(width: impl Into<Sizing>, height: impl Into<Sizing>) -> Self {
        Self { width: width.into(), height: height.into() }
    }

    /// Width and height are the same
    pub fn square(size: Sizing) -> Self {
        Self { width: size, height: size }
    }
}

/// Layout, paint, and text style properties for a widget.
///
/// `Style` is a trait rather than a struct so that animated styles, theme-driven
/// styles, and other dynamic implementations can exist without coupling to
/// [`WidgetStyle`]'s concrete fields. The `to_taffy` method has a blanket default
/// implementation that works for any implementor.
///
/// For most widgets, use the [`WidgetStyle`] builder.
pub trait Style: Send + Sync {
    fn display(&self) -> Display;
    fn sizing(&self) -> SizingAxes;
    fn padding(&self) -> Edges<Dimension>;
    fn margin(&self) -> Edges<Dimension>;
    fn gap(&self) -> GapStyles;
    fn overflow(&self) -> OverflowAxes;
    fn position(&self) -> Position;
    fn flex(&self) -> FlexProps;
    /// Per-child cross-axis alignment override. `None` defers to the parent's `align_items`.
    fn align_self(&self) -> Option<Align>;
    /// Aspect ratio as `(width, height)`, e.g. `(16.0, 9.0)`.
    fn aspect_ratio(&self) -> Option<(f32, f32)>;
    fn paint(&self) -> &PaintStyles;
    fn text(&self) -> TextStyles;

    /// Convert to a Taffy layout style.
    ///
    /// Text and paint fields are ignored - Taffy only consumes layout-relevant
    /// fields. `box-sizing` is always `border-box`.
    fn to_taffy(&self, viewport: ViewportSize) -> taffy::Style {
        let scale_x = viewport.width / 1280.0;
        let scale_y = viewport.height / 720.0;

        let sizing = self.sizing();

        let (w_size, w_min, w_max) = sizing.width.to_taffy_x(scale_x);
        let (h_size, h_min, h_max) = sizing.height.to_taffy_y(scale_y);

        let position = self.position();
        let display = self.display();
        let flex = self.flex();
        let gap = self.gap();
        let overflow = self.overflow();

        taffy::Style {
            display: match display {
                Display::Flex { .. } => taffy::Display::Flex,
                Display::None => taffy::Display::None,
            },
            box_sizing: taffy::BoxSizing::BorderBox,
            overflow: taffy::Point {
                x: overflow.x.to_taffy(),
                y: overflow.y.to_taffy(),
            },
            position: match position {
                Position::Absolute { offset: _ } => taffy::Position::Absolute,
                Position::Relative => taffy::Position::Relative,
            },
            inset: match position {
                Position::Absolute { offset } => taffy::Rect {
                    top: offset.top.to_lpa_y(scale_y),
                    bottom: offset.bottom.to_lpa_y(scale_y),
                    left: offset.left.to_lpa_x(scale_x),
                    right: offset.right.to_lpa_x(scale_x),
                },
                Position::Relative => {
                    taffy::Rect::auto() // Doesn't matter for relative
                }
            },
            size: taffy::Size {
                width: w_size,
                height: h_size,
            },
            min_size: taffy::Size {
                width: w_min,
                height: h_min,
            },
            max_size: taffy::Size {
                width: w_max,
                height: h_max,
            },
            aspect_ratio: self.aspect_ratio().map(|(x, y)| x / y),
            margin: self.margin().to_taffy_margin(scale_x, scale_y),
            padding: self.padding().to_taffy_padding(scale_x, scale_y),
            border: self.paint().border.to_taffy_border(),
            align_items: match display {
                Display::Flex {
                    align_items,
                    direction: _,
                    justify_content: _,
                } => Some(match align_items {
                    Align::Baseline => taffy::AlignItems::Baseline,
                    Align::Center => taffy::AlignItems::Center,
                    Align::End => taffy::AlignItems::End,
                    Align::Start => taffy::AlignItems::Start,
                    Align::Stretch => taffy::AlignItems::Stretch,
                }),
                _ => None,
            },
            align_self: self.align_self().map(|a| match a {
                Align::Baseline => taffy::AlignSelf::Baseline,
                Align::Center => taffy::AlignSelf::Center,
                Align::End => taffy::AlignSelf::End,
                Align::Start => taffy::AlignSelf::Start,
                Align::Stretch => taffy::AlignSelf::Stretch,
            }),
            justify_content: match display {
                Display::Flex {
                    justify_content,
                    direction: _,
                    align_items: _,
                } => Some(match justify_content {
                    Justify::Center => taffy::JustifyContent::Center,
                    Justify::End => taffy::JustifyContent::End,
                    Justify::SpaceBetween => taffy::JustifyContent::SpaceBetween,
                    Justify::SpaceAround => taffy::JustifyContent::SpaceAround,
                    Justify::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
                    Justify::Start => taffy::JustifyContent::Start,
                }),
                Display::None => None,
            },
            gap: taffy::Size {
                width: gap.column_gap.to_lp_x(scale_x),
                height: gap.row_gap.to_lp_y(scale_y),
            },
            flex_direction: match display {
                Display::Flex {
                    direction,
                    align_items: _,
                    justify_content: _,
                } => match direction {
                    FlexDirection::Column => taffy::FlexDirection::Column,
                    FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
                    FlexDirection::Row => taffy::FlexDirection::Row,
                    FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
                },
                _ => {
                    taffy::FlexDirection::default() // Doesn't matter here
                }
            },
            flex_basis: flex
                .flex_basis
                .map(Dimension::to_flex_basis)
                .unwrap_or(taffy::Dimension::auto()),
            flex_grow: flex.flex_grow.unwrap_or(0.0),
            flex_shrink: flex.flex_shrink.unwrap_or(1.0),
            ..Default::default()
        }
    }
}

/// Standard concrete [`Style`] implementation with a builder API.
///
/// Construct via [`WidgetStyle::new`], then chain `with_*` methods.
///
/// ```rust,ignore
/// let style = WidgetStyle::new()
///     .with_padding(Dimension::Px(16.0))
///     .with_background(Background::color(Color::hex("#1a1a2e")))
///     .with_border_radius(8.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetStyle {
    display: Display,
    sizing: SizingAxes,
    padding: Edges<Dimension>,
    margin: Edges<Dimension>,
    gap: GapStyles,
    overflow: OverflowAxes,
    position: Position,
    flex: FlexProps,
    align_self: Option<Align>,
    aspect_ratio: Option<(f32, f32)>,
    paint: PaintStyles,
    text: TextStyles,
}

impl Style for WidgetStyle {
    fn display(&self) -> Display {
        self.display
    }

    fn sizing(&self) -> SizingAxes {
        self.sizing
    }

    fn padding(&self) -> Edges<Dimension> {
        self.padding.clone()
    }

    fn margin(&self) -> Edges<Dimension> {
        self.margin.clone()
    }

    fn gap(&self) -> GapStyles {
        self.gap
    }

    fn overflow(&self) -> OverflowAxes {
        self.overflow
    }

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn flex(&self) -> FlexProps {
        self.flex
    }

    fn align_self(&self) -> Option<Align> {
        self.align_self
    }

    fn aspect_ratio(&self) -> Option<(f32, f32)> {
        self.aspect_ratio
    }

    fn paint(&self) -> &PaintStyles {
        &self.paint
    }

    fn text(&self) -> TextStyles {
        self.text.clone()
    }
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetStyle {
    /// Create a `WidgetStyle` with sensible defaults:
    /// flex row, no padding/margin/gap, transparent background, black 16px text.
    pub fn new() -> Self {
        Self {
            display: Display::default(),
            sizing: SizingAxes::default(),
            padding: Edges::all(Dimension::Px(0.0)),
            margin: Edges::all(Dimension::Px(0.0)),
            gap: GapStyles {
                column_gap: Dimension::Px(0.0),
                row_gap: Dimension::Px(0.0),
            },
            overflow: OverflowAxes {
                x: Overflow::Visible,
                y: Overflow::Visible,
            },
            position: Position::Relative,
            flex: FlexProps {
                flex_grow: None,
                flex_shrink: None,
                flex_basis: None,
            },
            align_self: None,
            aspect_ratio: None,
            paint: PaintStyles {
                background: Background::color(Colors::TRANSPARENT),
                border: BorderStyle::new(Edges::all(BorderEdge {
                    width: 0.0,
                    texture: Texture::Color(Colors::TRANSPARENT),
                }), Corners::all(0.0)),
                shadows: vec![],
                opacity: 1.0,
            },
            text: TextStyles {
                text_paint: Texture::Color(Colors::BLACK),
                font_size: FontSize::Px(16.0),
                font_weight: FontWeight::Regular,
                font_family: "system",
                text_align: TextAlign::Start,
                line_height: 1.5,
            },
        }
    }

    pub fn with_display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }

    pub fn with_sizing(mut self, sizing: SizingAxes) -> Self {
        self.sizing = sizing;
        self
    }

    pub fn with_width(mut self, width: Sizing) -> Self {
        self.sizing.width = width;
        self
    }

    pub fn with_height(mut self, height: Sizing) -> Self {
        self.sizing.height = height;
        self
    }

    /// Set all four padding edges to the same value.
    pub fn with_padding(mut self, padding: Dimension) -> Self {
        self.padding = Edges::all(padding);
        self
    }

    pub fn with_padding_x(mut self, padding: Dimension) -> Self {
        self.padding.right = padding;
        self.padding.left = padding;
        self
    }

    pub fn with_padding_y(mut self, padding: Dimension) -> Self {
        self.padding.top = padding;
        self.padding.bottom = padding;
        self
    }

    pub fn with_padding_top(mut self, padding: Dimension) -> Self {
        self.padding.top = padding;
        self
    }

    pub fn with_padding_bottom(mut self, padding: Dimension) -> Self {
        self.padding.bottom = padding;
        self
    }

    pub fn with_padding_left(mut self, padding: Dimension) -> Self {
        self.padding.left = padding;
        self
    }

    pub fn with_padding_right(mut self, padding: Dimension) -> Self {
        self.padding.right = padding;
        self
    }

    /// Set all four margin edges to the same value.
    pub fn with_margin(mut self, margin: Dimension) -> Self {
        self.margin = Edges::all(margin);
        self
    }

    pub fn with_margin_x(mut self, margin: Dimension) -> Self {
        self.margin.right = margin;
        self.margin.left = margin;
        self
    }

    pub fn with_margin_y(mut self, margin: Dimension) -> Self {
        self.margin.top = margin;
        self.margin.bottom = margin;
        self
    }

    pub fn with_margin_top(mut self, margin: Dimension) -> Self {
        self.margin.top = margin;
        self
    }

    pub fn with_margin_bottom(mut self, margin: Dimension) -> Self {
        self.margin.bottom = margin;
        self
    }

    pub fn with_margin_left(mut self, margin: Dimension) -> Self {
        self.margin.left = margin;
        self
    }

    pub fn with_margin_right(mut self, margin: Dimension) -> Self {
        self.margin.right = margin;
        self
    }

    pub fn with_gap(mut self, gap: GapStyles) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_column_gap(mut self, gap: Dimension) -> Self {
        self.gap.column_gap = gap;
        self
    }

    pub fn with_row_gap(mut self, gap: Dimension) -> Self {
        self.gap.row_gap = gap;
        self
    }

    pub fn with_overflow(mut self, overflow: OverflowAxes) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn with_overflow_x(mut self, overflow: Overflow) -> Self {
        self.overflow.x = overflow;
        self
    }

    pub fn with_overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow.y = overflow;
        self
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn with_flex_grow(mut self, flex_grow: Option<f32>) -> Self {
        self.flex.flex_grow = flex_grow;
        self
    }

    pub fn with_flex_shrink(mut self, flex_shrink: Option<f32>) -> Self {
        self.flex.flex_shrink = flex_shrink;
        self
    }

    pub fn with_flex_basis(mut self, flex_basis: Option<Dimension>) -> Self {
        self.flex.flex_basis = flex_basis;
        self
    }

    pub fn with_align_self(mut self, align_self: Option<Align>) -> Self {
        self.align_self = align_self;
        self
    }

    pub fn with_aspect_ratio(mut self, aspect_ratio: Option<(f32, f32)>) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub fn with_background(mut self, background: Background) -> Self {
        self.paint.background = background;
        self
    }

    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.paint.border = border;
        self
    }

    pub fn with_border_x(mut self, border_x: BorderEdge) -> Self {
        self.paint.border.edges.left = border_x.clone();
        self.paint.border.edges.right = border_x;
        self
    }

    pub fn with_border_y(mut self, border_y: BorderEdge) -> Self {
        self.paint.border.edges.top = border_y.clone();
        self.paint.border.edges.bottom = border_y;
        self
    }

    pub fn with_border_top(mut self, border: BorderEdge) -> Self {
        self.paint.border.edges.top = border;
        self
    }

    pub fn with_border_bottom(mut self, border: BorderEdge) -> Self {
        self.paint.border.edges.bottom = border;
        self
    }

    pub fn with_border_left(mut self, border: BorderEdge) -> Self {
        self.paint.border.edges.left = border;
        self
    }

    pub fn with_border_right(mut self, border: BorderEdge) -> Self {
        self.paint.border.edges.right = border;
        self
    }

    /// Set a uniform border radius on all corners.
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.paint.border.radius = Corners::all(radius);
        self
    }

    pub fn with_border_radius_top(mut self, radius: f32) -> Self {
        self.paint.border.radius.top_left = radius;
        self.paint.border.radius.top_right = radius;
        self
    }

    pub fn with_border_radius_bottom(mut self, radius: f32) -> Self {
        self.paint.border.radius.bottom_left = radius;
        self.paint.border.radius.bottom_right = radius;
        self
    }

    pub fn with_border_radius_left(mut self, radius: f32) -> Self {
        self.paint.border.radius.top_left = radius;
        self.paint.border.radius.bottom_left = radius;
        self
    }

    pub fn with_border_radius_right(mut self, radius: f32) -> Self {
        self.paint.border.radius.top_right = radius;
        self.paint.border.radius.bottom_right = radius;
        self
    }

    pub fn with_border_radii(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_left: f32,
        bottom_right: f32,
    ) -> Self {
        self.paint.border.radius =
            Corners::individual(top_left, top_right, bottom_left, bottom_right);
        self
    }

    pub fn with_top_left(mut self, top_left: f32) -> Self {
        self.paint.border.radius.top_left = top_left;
        self
    }

    pub fn with_top_right(mut self, top_right: f32) -> Self {
        self.paint.border.radius.top_right = top_right;
        self
    }

    pub fn with_bottom_left(mut self, bottom_left: f32) -> Self {
        self.paint.border.radius.bottom_left = bottom_left;
        self
    }

    pub fn with_bottom_right(mut self, bottom_right: f32) -> Self {
        self.paint.border.radius.bottom_right = bottom_right;
        self
    }

    /// Append a shadow. Multiple shadows are rendered back-to-front.
    pub fn with_shadow(mut self, shadow: BoxShadow) -> Self {
        self.paint.shadows.push(shadow);
        self
    }

    pub fn with_shadows(mut self, shadows: Vec<BoxShadow>) -> Self {
        self.paint.shadows.extend(shadows);
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.paint.opacity = opacity;
        self
    }

    pub fn with_text_paint(mut self, paint: Texture) -> Self {
        self.text.text_paint = paint;
        self
    }

    pub fn with_font_size(mut self, font_size: FontSize) -> Self {
        self.text.font_size = font_size;
        self
    }

    pub fn with_font_weight(mut self, font_weight: FontWeight) -> Self {
        self.text.font_weight = font_weight;
        self
    }

    pub fn with_font_family(mut self, font_family: &'static str) -> Self {
        self.text.font_family = font_family;
        self
    }

    pub fn with_text_align(mut self, text_align: TextAlign) -> Self {
        self.text.text_align = text_align;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.text.line_height = line_height;
        self
    }
}

/// A 2D point in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}