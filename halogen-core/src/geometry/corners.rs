#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Corners<T> {
    top_left: T,
    top_right: T,
    bottom_left: T,
    bottom_right: T,
}

impl<T: Clone> Corners<T> {
    pub fn all(value: T) -> Self {
        Self {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_left: value.clone(),
            bottom_right: value,
        }
    }
}

impl<T> Corners<T> {
    pub fn new(top_left: T, top_right: T, bottom_left: T, bottom_right: T) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
}

impl<T: Default> Default for Corners<T> {
    fn default() -> Self {
        Self {
            top_left: Default::default(),
            top_right: Default::default(),
            bottom_left: Default::default(),
            bottom_right: Default::default(),
        }
    }
}
