#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axes<T> {
    pub horizontal: T,
    pub vertical: T,
}

impl<T: Clone> Axes<T> {
    pub fn both(value: T) -> Self {
        Self::new(value.clone(), value)
    }
}

impl<T> Axes<T> {
    pub const fn new(horizontal: T, vertical: T) -> Self {
        Self { horizontal, vertical }
    }
}

impl<T: Default> Default for Axes<T> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}