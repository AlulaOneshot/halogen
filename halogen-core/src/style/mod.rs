use crate::color::{Color, ConicGradient, LinearGradient, MeshGradient, RadialGradient};
use crate::geometry::{
    Axes, Corners, Edges, FlexBasisLength, Length, Pixels, Point, Sizing, UnboundedLength,
};

mod display;

pub use display::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Color(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    ConicGradient(ConicGradient),
    MeshGradient(MeshGradient),
    //TODO: Image
}

impl Default for Paint {
    fn default() -> Self {
        Self::Color(Color::default())
    }
}

// =========
// Box Model
// =========

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BorderStyles {
    pub width: f32,
    pub paint: Paint,
    pub radius: Corners<f32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoxModelStyles {
    pub content: Axes<Sizing>,
    pub padding: Edges<Length>,
    pub border: BorderStyles,
    pub margin: Edges<Length>,
}

// ========
// Overflow
// ========

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    /// Behaves as hidden, serves as a marker for widgets that implement scrolling.
    Scroll,
}

// ========
// Position
// ========

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Position {
    /// The element follows the normal flow of layout
    #[default]
    Flow,
    /// The element is removed from the normal flow of the layout. Positioned by distance from edges.
    Absolute(Edges<Length>),
}

// =====
// Paint
// =====

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoxShadow {
    /// (X, Y) offset from the widget.
    pub offset: Point<UnboundedLength>,
    /// Gaussian blur radius in pixels
    pub blur: Pixels,
    /// Extra expansion of the shadow beyond the widget's bounds.
    pub spread: Pixels,
    pub color: Color,
    pub inset: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaintStyles {
    pub background: Paint,
    pub shadows: Vec<BoxShadow>,
    pub opacity: f32,
    pub visible: bool,
}

// ====
// Text
// ====

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextAlignment {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyles {
    pub paint: Paint,
    pub font_size: UnboundedLength,
    pub font_weight: FontWeight,
    pub font_family: String,
    pub align: TextAlignment,
    pub line_height: f32,
}

// ==============================
// Flex Properties (for children)
// ==============================

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlexProperties {
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<FlexBasisLength>,
}

// ==============================
// Grid Properties (for children)
// ==============================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridPlacement {
    #[default]
    Auto,
    Line(i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GridProperties {
    pub grid_row_start: GridPlacement,
    pub grid_row_end: GridPlacement,
    pub grid_column_start: GridPlacement,
    pub grid_column_end: GridPlacement,
}

// ========
// Viewport
// ========

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewportSize {
    pub width: Pixels,
    pub height: Pixels,
}

// ================
// Layout Direction
// ================

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LayoutDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

// =====
// Style
// =====

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub display: DisplayStyles,
    pub box_model: BoxModelStyles,
    /// Controls the gap between elements across axes.
    pub gap: Axes<Length>,
    /// Controls the children overflow behavior across axes.
    pub overflow: Axes<Overflow>,
    pub position: Position,
    /// Controls this childs flex properties. Only effective if the parent is a flex layout
    pub flex: FlexProperties,
    pub grid: GridProperties,
    pub align_self: AlignSelf,
    pub justify_self: AlignSelf,
    pub aspect_ratio: Option<(f32, f32)>,
    pub paint: PaintStyles,
    pub text: TextStyles,
    pub direction: LayoutDirection
}

impl Style {
    pub(crate) fn to_taffy(&self, viewport: ViewportSize) -> taffy::Style {
        let vw = viewport.width.0;
        let vh = viewport.height.0;

        let scale_units_scale = (f32::max(vw, vh) / 1280.0) * 50.0;

        taffy::Style {
            display: match &self.display {
                DisplayStyles::Flex(_) => taffy::Display::Flex,
                DisplayStyles::Grid(_) => taffy::Display::Grid,
                DisplayStyles::None => taffy::Display::None
            },
            item_is_table: false,
            item_is_replaced: false,
            box_sizing: taffy::BoxSizing::BorderBox,
            direction: match &self.direction {
                LayoutDirection::LeftToRight => taffy::Direction::Ltr,
                LayoutDirection::RightToLeft => taffy::Direction::Rtl
            },
            overflow: taffy::Point {
                x: match &self.overflow.horizontal {
                    Overflow::Visible => taffy::Overflow::Visible,
                    Overflow::Hidden => taffy::Overflow::Hidden,
                    Overflow::Scroll => taffy::Overflow::Hidden
                },
                y: match &self.overflow.vertical {
                    Overflow::Visible => taffy::Overflow::Visible,
                    Overflow::Hidden => taffy::Overflow::Hidden,
                    Overflow::Scroll => taffy::Overflow::Hidden
                }
            },
            scrollbar_width: 0.0,
            float: taffy::Float::None,
            clear: taffy::Clear::None,
            position: match &self.position {
                Position::Absolute(_) => {
                    taffy::Position::Absolute
                },
                Position::Flow => {
                    taffy::Position::Relative
                }
            },
            inset: match &self.position {
                Position::Absolute(edges) => {
                    taffy::Rect {
                        top: edges.top.to_lpa(scale_units_scale, Some(vh)),
                        bottom: edges.bottom.to_lpa(scale_units_scale, Some(vh)),
                        left: edges.left.to_lpa(scale_units_scale, Some(vw)),
                        right: edges.right.to_lpa(scale_units_scale, Some(vw))
                    }
                },
                Position::Flow => {
                    taffy::Rect::zero()
                }
            },
            size: taffy::Size {
                height: match &self.box_model.content.vertical {
                    Sizing::Auto => taffy::Dimension::auto(),
                    Sizing::Exact(len) => len.to_lpa(scale_units_scale, Some(vh)).into(),
                    _ => taffy::Dimension::auto()
                },
                width: match &self.box_model.content.horizontal {
                    Sizing::Auto => taffy::Dimension::auto(),
                    Sizing::Exact(len) => len.to_lpa(scale_units_scale, Some(vw)).into(),
                    _ => taffy::Dimension::auto()
                },
            },
            min_size: taffy::Size {
                height: match &self.box_model.content.vertical {
                    Sizing::MinMax(min, _) => min.to_lpa(scale_units_scale, Some(vh)).into(),
                    _ => taffy::Dimension::auto()
                },
                width: match &self.box_model.content.horizontal {
                    Sizing::MinMax(min, _) => min.to_lpa(scale_units_scale, Some(vw)).into(),
                    _ => taffy::Dimension::auto()
                }
            },
            max_size: taffy::Size {
                height: match &self.box_model.content.vertical {
                    Sizing::MinMax(_, max) => max.to_lpa(scale_units_scale, Some(vh)).into(),
                    _ => taffy::Dimension::auto()
                },
                width: match &self.box_model.content.horizontal {
                    Sizing::MinMax(_, max) => max.to_lpa(scale_units_scale, Some(vw)).into(),
                    _ => taffy::Dimension::auto()
                }
            },
            aspect_ratio: match self.aspect_ratio {
                Some((one, two)) => Some(one / two),
                None => None
            },
            margin: taffy::Rect {
                top: self.box_model.margin.top.to_lpa(scale_units_scale, Some(vh)),
                bottom: self.box_model.margin.bottom.to_lpa(scale_units_scale, Some(vh)),
                left: self.box_model.margin.left.to_lpa(scale_units_scale, Some(vw)),
                right: self.box_model.margin.right.to_lpa(scale_units_scale, Some(vw))
            },
            padding: taffy::Rect {
                top: self.box_model.padding.top.to_lp(scale_units_scale, Some(vh)),
                bottom: self.box_model.padding.bottom.to_lp(scale_units_scale, Some(vh)),
                left: self.box_model.padding.left.to_lp(scale_units_scale, Some(vw)),
                right: self.box_model.padding.right.to_lp(scale_units_scale, Some(vw))
            },
            border: taffy::Rect {
                top: taffy::LengthPercentage::length(self.box_model.border.width),
                bottom: taffy::LengthPercentage::length(self.box_model.border.width),
                left: taffy::LengthPercentage::length(self.box_model.border.width),
                right: taffy::LengthPercentage::length(self.box_model.border.width)
            },
            align_items: match &self.display {
                DisplayStyles::Flex(flex) => {
                    Some(flex.align_items.to_taffy())
                },
                DisplayStyles::Grid(grid) => {
                    Some(grid.align_items.to_taffy())
                },
                DisplayStyles::None => None
            },
            align_self: Some(self.align_self.to_taffy()),
            justify_items: match &self.display {
                DisplayStyles::Flex(_) => None,
                DisplayStyles::Grid(grid) => {
                    Some(grid.justify_items.to_taffy())
                },
                DisplayStyles::None => None
            },
            justify_self: Some(self.justify_self.to_taffy()),
            justify_content: match &self.display {
                DisplayStyles::Flex(flex) => {
                    Some(flex.justify_content.to_taffy())
                },
                DisplayStyles::Grid(grid) => {
                    Some(grid.justify_content.to_taffy())
                },
                DisplayStyles::None => None
            },
            align_content: match &self.display {
                DisplayStyles::Flex(flex) => {
                    Some(flex.align_content.to_taffy())
                },
                DisplayStyles::Grid(grid) => {
                    Some(grid.align_content.to_taffy())
                },
                DisplayStyles::None => None
            },
            gap: taffy::Size {
                width: self.gap.horizontal.to_lp(scale_units_scale, Some(vw)),
                height: self.gap.vertical.to_lp(scale_units_scale, Some(vh))
            },
            text_align: taffy::TextAlign::Auto,
            flex_direction: match &self.display {
                DisplayStyles::Flex(flex) => {
                    flex.direction.to_taffy()
                },
                _ => taffy::FlexDirection::Row
            },
            flex_wrap: match &self.display {
                DisplayStyles::Flex(flex) => {
                    flex.wrap.to_taffy()
                },
                _ => taffy::FlexWrap::NoWrap
            },
            flex_basis: match &self.flex.flex_basis {
                Some(basis) => {
                    basis.to_dim(scale_units_scale)
                },
                None => taffy::Dimension::auto()
            },
            flex_grow: self.flex.flex_grow.unwrap_or(0.0),
            flex_shrink: self.flex.flex_shrink.unwrap_or(1.0),
            grid_template_rows: match &self.display {
                DisplayStyles::Grid(grid) => {
                    grid.template_rows.iter().map(grid_track_to_template).collect()
                },
                _ => vec![]
            },
            grid_template_columns: match &self.display {
                DisplayStyles::Grid(grid) => {
                    grid.template_columns.iter().map(grid_track_to_template).collect()
                },
                _ => vec![]
            },
            grid_auto_rows: match &self.display {
                DisplayStyles::Grid(grid) => vec![grid_track_to_taffy(&grid.auto_rows)],
                _ => vec![]
            },
            grid_auto_columns: match &self.display {
                DisplayStyles::Grid(grid) => vec![grid_track_to_taffy(&grid.auto_columns)],
                _ => vec![]
            },
            grid_auto_flow: match &self.display {
                DisplayStyles::Grid(grid) => match grid.auto_flow {
                    GridAutoFlow::Row => taffy::GridAutoFlow::Row,
                    GridAutoFlow::Column => taffy::GridAutoFlow::Column,
                    GridAutoFlow::RowDense => taffy::GridAutoFlow::RowDense,
                    GridAutoFlow::ColumnDense => taffy::GridAutoFlow::ColumnDense,
                },
                _ => taffy::GridAutoFlow::Row
            },
            grid_column: {
                let start = self.grid.grid_column_start;
                let end = self.grid.grid_column_end;

                taffy::Line {
                    start: match start {
                        GridPlacement::Auto => taffy::GridPlacement::Auto,
                        GridPlacement::Line(l) => <taffy::GridPlacement as taffy::prelude::TaffyGridLine>::from_line_index(l)
                    },
                    end: match end {
                        GridPlacement::Auto => taffy::GridPlacement::Auto,
                        GridPlacement::Line(l) => <taffy::GridPlacement as taffy::prelude::TaffyGridLine>::from_line_index(l)
                    }
                }
            },
            grid_row: {
                let start = self.grid.grid_row_start;
                let end = self.grid.grid_row_end;

                taffy::Line {
                    start: match start {
                        GridPlacement::Auto => taffy::GridPlacement::Auto,
                        GridPlacement::Line(l) => <taffy::GridPlacement as taffy::prelude::TaffyGridLine>::from_line_index(l)
                    },
                    end: match end {
                        GridPlacement::Auto => taffy::GridPlacement::Auto,
                        GridPlacement::Line(l) => <taffy::GridPlacement as taffy::prelude::TaffyGridLine>::from_line_index(l)
                    }
                }
            },
            ..Default::default()
        }
    }
}

fn grid_track_to_template(track: &GridTrack) -> taffy::GridTemplateComponent<String> {
    match track {
        GridTrack::Repeat(count, tracks) => taffy::GridTemplateComponent::Repeat(
            taffy::GridTemplateRepetition {
                count:      repeat_count_to_taffy(count),
                tracks:     tracks.iter().map(grid_track_to_taffy).collect(),
                line_names: vec![], // we don'taffy support named lines
            }
        ),
        other => taffy::GridTemplateComponent::Single(grid_track_to_taffy(other)),
    }
}

fn repeat_count_to_taffy(c: &RepeatCount) -> taffy::RepetitionCount {
    match c {
        RepeatCount::Count(n)  => taffy::RepetitionCount::Count(*n as u16),
        RepeatCount::AutoFill  => taffy::RepetitionCount::AutoFill,
        RepeatCount::AutoFit   => taffy::RepetitionCount::AutoFit,
    }
}

fn grid_track_to_taffy(track: &GridTrack) -> taffy::TrackSizingFunction {
    match track {
        GridTrack::Auto                => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::auto(), max: taffy::MaxTrackSizingFunction::auto() },
        GridTrack::Pixels(px)          => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::length(px.0), max: taffy::MaxTrackSizingFunction::length(px.0) },
        GridTrack::Percentage(pct)     => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::percent(pct.0 / 100.0), max: taffy::MaxTrackSizingFunction::percent(pct.0 / 100.0) },
        GridTrack::FractionalUnits(fr) => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::auto(), max: taffy::MaxTrackSizingFunction::fr(fr.0) },
        GridTrack::MinContent          => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::min_content(), max: taffy::MaxTrackSizingFunction::min_content() },
        GridTrack::MaxContent          => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::max_content(), max: taffy::MaxTrackSizingFunction::max_content() },
        GridTrack::MinMax(min, max)    => taffy::TrackSizingFunction { min: grid_track_min_to_taffy(min), max: grid_track_max_to_taffy(max) },
        GridTrack::ScaleUnits(_)       => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::auto(), max: taffy::MaxTrackSizingFunction::auto() },
        GridTrack::Repeat(_, _)        => taffy::TrackSizingFunction { min: taffy::MinTrackSizingFunction::auto(), max: taffy::MaxTrackSizingFunction::auto() },
    }
}

fn grid_track_min_to_taffy(m: &GridTrackMin) -> taffy::MinTrackSizingFunction {
    match m {
        GridTrackMin::Auto           => taffy::MinTrackSizingFunction::auto(),
        GridTrackMin::Pixels(px)     => taffy::MinTrackSizingFunction::length(px.0),
        GridTrackMin::Percentage(pct)=> taffy::MinTrackSizingFunction::percent(pct.0 / 100.0),
        GridTrackMin::ScaleUnits(_)  => taffy::MinTrackSizingFunction::auto(),
        GridTrackMin::MinContent     => taffy::MinTrackSizingFunction::min_content(),
        GridTrackMin::MaxContent     => taffy::MinTrackSizingFunction::max_content(),
    }
}

fn grid_track_max_to_taffy(m: &GridTrackMax) -> taffy::MaxTrackSizingFunction {
    match m {
        GridTrackMax::Auto              => taffy::MaxTrackSizingFunction::auto(),
        GridTrackMax::Pixels(px)        => taffy::MaxTrackSizingFunction::length(px.0),
        GridTrackMax::Percentage(pct)   => taffy::MaxTrackSizingFunction::percent(pct.0 / 100.0),
        GridTrackMax::FractionalUnits(fr) => taffy::MaxTrackSizingFunction::fr(fr.0),
        GridTrackMax::ScaleUnits(_)     => taffy::MaxTrackSizingFunction::auto(),
        GridTrackMax::MinContent        => taffy::MaxTrackSizingFunction::min_content(),
        GridTrackMax::MaxContent        => taffy::MaxTrackSizingFunction::max_content(),
    }
}