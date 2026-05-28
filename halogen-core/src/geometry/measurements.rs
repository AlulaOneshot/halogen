use crate::geometry::{Percentage, Pixels, ScaleUnits};

#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq)]
pub enum Length {
    #[default]
    Auto,
    Pixels(Pixels),
    Percent(Percentage),
    ScaleUnits(ScaleUnits),
    ViewportPercent(Percentage),
}

impl Length {
    pub const ZERO: Self = Self::Pixels(Pixels::ZERO);
    pub const FULL: Self = Self::Percent(Percentage(100.0));
    pub const HALF: Self = Self::Percent(Percentage(50.0));
}

#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq)]
pub enum FlexBasisLength {
    #[default]
    Auto,
    Pixels(Pixels),
    Percent(Percentage),
    ScaleUnits(ScaleUnits),
}

impl FlexBasisLength {
    pub const ZERO: Self = Self::Pixels(Pixels::ZERO);
    pub const FULL: Self = Self::Percent(Percentage(100.0));
    pub const HALF: Self = Self::Percent(Percentage(50.0));
}

// Length Measurements with no reliable parent.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum UnboundedLength {
    Pixels(Pixels),
    ScaleUnits(ScaleUnits),
}

impl UnboundedLength {
    pub const ZERO: Self = Self::Pixels(Pixels::ZERO);
}

impl Default for UnboundedLength {
    fn default() -> Self {
        Self::Pixels(Pixels::ZERO)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq)]
pub enum Sizing {
    Exact(Length),
    MinMax(Length, Length),
    #[default]
    Auto
}

impl Sizing {
    pub const ZERO: Self = Self::Exact(Length::ZERO);
}