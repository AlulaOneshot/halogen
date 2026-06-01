use crate::geometry::Axes;

pub type MeasureFunc = Box<dyn Fn(MeasureInput) -> Axes<f32> + Send + Sync + 'static>;

pub struct MeasureInput {
    /// Dimensions already fixed by style (e.g. explicit width/height set).
    /// If `Some`, your measure should respect it rather than computing fresh.
    pub known_width:  Option<f32>,
    pub known_height: Option<f32>,
    /// The space taffy is offering this node.
    pub available_width:  AvailableSpace,
    pub available_height: AvailableSpace,
}

pub enum AvailableSpace {
    /// Specified amount of space.
    Definite(f32),
    /// Unlimited
    Infinite,
    /// Attempt to make it as small as possible.
    MinContent,
}