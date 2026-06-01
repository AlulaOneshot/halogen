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

    /// if `viewport_axis_length` is `None`, `Self::ViewportPercent` panics.
    pub(crate) fn to_lpa(&self, scale_units_scale: f32, viewport_axis_length: Option<f32>) -> taffy::LengthPercentageAuto {
        match self {
            Self::Auto => {
                taffy::LengthPercentageAuto::auto()
            },
            Self::Pixels(px) => {
                taffy::LengthPercentageAuto::length(px.0)
            },
            Self::Percent(percent) => {
                taffy::LengthPercentageAuto::percent(percent.0)
            },
            Self::ScaleUnits(scale) => {
                taffy::LengthPercentageAuto::length(scale.0 * scale_units_scale)
            },
            Self::ViewportPercent(percent) => {
                if let Some(length) = viewport_axis_length {
                    taffy::LengthPercentageAuto::length(length * (percent.0 / 100.0))
                } else {
                    panic!()
                }
            }
        }
    }

    /// if `viewport_axis_length` is `None`, `Self::ViewportPercent` panics.
    pub(crate) fn to_lp(&self, scale_units_scale: f32, viewport_axis_length: Option<f32>) -> taffy::LengthPercentage {
        match self {
            Self::Auto => {
                taffy::LengthPercentage::length(0.0)
            },
            Self::Pixels(px) => {
                taffy::LengthPercentage::length(px.0)
            },
            Self::Percent(percent) => {
                taffy::LengthPercentage::percent(percent.0)
            },
            Self::ScaleUnits(scale) => {
                taffy::LengthPercentage::length(scale.0 * scale_units_scale)
            },
            Self::ViewportPercent(percent) => {
                if let Some(length) = viewport_axis_length {
                    taffy::LengthPercentage::length(length / (percent.0 / 100.0))
                } else {
                    panic!()
                }
            }
        }
    }
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

    pub(crate) fn to_dim(self, scale_units_scale: f32) -> taffy::Dimension {
        match self {
            FlexBasisLength::Auto => taffy::Dimension::auto(),
            FlexBasisLength::Pixels(px) => taffy::Dimension::length(px.0),
            FlexBasisLength::Percent(percent) => taffy::Dimension::percent(percent.0),
            FlexBasisLength::ScaleUnits(scale) => taffy::Dimension::length(scale.0 * scale_units_scale),
        }
    }
}

// Length Measurements with no reliable parent.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum UnboundedLength {
    Pixels(Pixels),
    ScaleUnits(ScaleUnits),
}

impl UnboundedLength {
    pub const ZERO: Self = Self::Pixels(Pixels::ZERO);

    pub fn to_px(&self, scale_units_scale: f32) -> f32 {
        match self {
            Self::Pixels(px) => {
                px.0
            },
            Self::ScaleUnits(scale) => {
                scale.0 * scale_units_scale
            }
        }
    }
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