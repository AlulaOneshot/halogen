use crate::geometry::Axes;
use crate::geometry::point::Point;

pub struct Rect {
    pub origin: Point<f32>,
    pub size: Axes<f32>
}

impl Rect {
    pub fn new(origin: Point<f32>, size: Axes<f32>) -> Self {
        Self { origin, size }
    }
}

