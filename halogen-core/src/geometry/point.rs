#[derive(Debug, Copy, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Clone> Clone for Point<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

impl<T: Clone> Point<T> {
    pub fn both(v: T) -> Self {
        Self { x: v.clone(), y: v }
    }
}

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}