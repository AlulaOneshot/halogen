use crate::geometry::{GridFractionalUnits, Percentage, Pixels, ScaleUnits};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    pub(crate) fn to_taffy(self) -> taffy::FlexDirection {
        match self {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum JustifyContent {
    Start,
    End,
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl JustifyContent {
    pub(crate) fn to_taffy(self) -> taffy::JustifyContent {
        match self {
            JustifyContent::Start => taffy::JustifyContent::Start,
            JustifyContent::End => taffy::JustifyContent::End,
            JustifyContent::FlexStart => taffy::JustifyContent::FlexStart,
            JustifyContent::FlexEnd => taffy::JustifyContent::FlexEnd,
            JustifyContent::Center => taffy::JustifyContent::Center,
            JustifyContent::Stretch => taffy::JustifyContent::Stretch,
            JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
            JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    #[default]
    Stretch,
}

impl AlignItems {
    pub(crate) fn to_taffy(self) -> taffy::AlignItems {
        match self {
            AlignItems::Start => taffy::AlignItems::Start,
            AlignItems::End => taffy::AlignItems::End,
            AlignItems::FlexStart => taffy::AlignItems::FlexStart,
            AlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::Baseline => taffy::AlignItems::Baseline,
            AlignItems::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum AlignSelf {
    #[default]
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl AlignSelf {
    pub(crate) fn to_taffy(self) -> taffy::AlignSelf {
        match self {
            AlignSelf::Start => taffy::AlignSelf::Start,
            AlignSelf::End => taffy::AlignSelf::End,
            AlignSelf::FlexStart => taffy::AlignSelf::FlexStart,
            AlignSelf::FlexEnd => taffy::AlignSelf::FlexEnd,
            AlignSelf::Center => taffy::AlignSelf::Center,
            AlignSelf::Baseline => taffy::AlignSelf::Baseline,
            AlignSelf::Stretch => taffy::AlignSelf::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl AlignContent {
    pub(crate) fn to_taffy(self) -> taffy::AlignContent {
        match self {
            AlignContent::Start => taffy::AlignContent::Start,
            AlignContent::End => taffy::AlignContent::End,
            AlignContent::FlexStart => taffy::AlignContent::FlexStart,
            AlignContent::FlexEnd => taffy::AlignContent::FlexEnd,
            AlignContent::Center => taffy::AlignContent::Center,
            AlignContent::Stretch => taffy::AlignContent::Stretch,
            AlignContent::SpaceBetween => taffy::AlignContent::SpaceBetween,
            AlignContent::SpaceEvenly => taffy::AlignContent::SpaceEvenly,
            AlignContent::SpaceAround => taffy::AlignContent::SpaceAround,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

impl FlexWrap {
    pub(crate) fn to_taffy(self) -> taffy::FlexWrap {
        match self {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct DisplayFlexStyles {
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/flex-direction
    /// Controls how items are placed in the flex container and by setting the main axis and direction.
    pub direction: FlexDirection,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-content
    /// Controls how space is distributed between and around content items along the main axis of a flex or grid container.
    pub justify_content: JustifyContent,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-content
    /// Controls the distribution between and around items along the cross axis of a flex or grid container when there is extra space in the cross axis.
    pub align_content: AlignContent,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-items
    /// Controls the default `align_self` for all children.
    pub align_items: AlignItems,
    pub wrap: FlexWrap,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense, // Row but with the dense packing algorithm
    ColumnDense, // Column but with the dense packing algorithm
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridTrackMin {
    #[default]
    Auto,
    Pixels(Pixels),
    ScaleUnits(ScaleUnits),
    Percentage(Percentage),
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridTrackMax {
    #[default]
    Auto,
    Pixels(Pixels),
    ScaleUnits(ScaleUnits),
    Percentage(Percentage),
    FractionalUnits(GridFractionalUnits),
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum RepeatCount {
    Count(usize),
    #[default]
    AutoFill,
    AutoFit,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum GridTrack {
    #[default]
    Auto,
    Pixels(Pixels),
    ScaleUnits(ScaleUnits),
    Percentage(Percentage),
    /// Functions as a fraction of the grid space i.e
    /// in a 500px high grid, if columns were defined as 1fr, 3fr, 1fr,
    /// they would be 100px, 300px, and 100px respectively
    FractionalUnits(GridFractionalUnits),
    MinContent, // Minimum content size
    MaxContent, // Maximum content size
    MinMax(GridTrackMin, GridTrackMax),
    Repeat(RepeatCount, Vec<GridTrack>), // Repeat the provided pattern ([GridTrack]) for RepeatCount iterations
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayGridStyles {
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-content
    /// Controls how space is distributed between and around content items along the main axis of a flex or grid container.
    pub justify_content: JustifyContent,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-items
    /// Controls the default `justifySelf` for all children.
    pub justify_items: AlignItems,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-content
    /// Controls the distribution between and around items along the cross axis of a flex or grid container when there is extra space in the cross axis.
    pub align_content: AlignContent,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-items
    /// Controls the default `align_self` for all children.
    pub align_items: AlignItems,

    pub template_rows: Vec<GridTrack>,
    pub template_columns: Vec<GridTrack>,

    pub auto_rows: GridTrack,
    pub auto_columns: GridTrack,

    pub auto_flow: GridAutoFlow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayStyles {
    Flex(DisplayFlexStyles),
    Grid(DisplayGridStyles),
    None  // Element takes no space in layout. Not rendered.
}

impl Default for DisplayStyles {
    fn default() -> Self {
        Self::Flex(Default::default())
    }
}