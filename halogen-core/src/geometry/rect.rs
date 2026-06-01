use crate::geometry::point::Point;
use crate::geometry::Axes;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub origin: Point<f32>,
    pub size: Axes<f32>,
}

impl Rect {
    pub fn new(origin: Point<f32>, size: Axes<f32>) -> Self {
        Self { origin, size }
    }

    pub fn zero() -> Self {
        Self {
            origin: Point::both(0.0),
            size: Axes::both(0.0),
        }
    }
}

