#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Edges<T> {
    top: T,
    bottom: T,
    left: T,
    right: T,
}

impl<T: Clone> Edges<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            bottom: value.clone(),
            left: value.clone(),
            right: value,
        }
    }

    pub fn hv(horizontal: T, vertical: T) -> Self {
        Self {
            top: vertical.clone(),
            bottom: vertical.clone(),
            left: horizontal.clone(),
            right: horizontal,
        }
    }
}

impl<T> Edges<T> {
    pub fn new(top: T, bottom: T, left: T, right: T) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }
}

impl<T: Default> Default for Edges<T> {
    fn default() -> Self {
        Self {
            top: Default::default(),
            bottom: Default::default(),
            left: Default::default(),
            right: Default::default(),
        }
    }
}
