pub struct Axes<T> {
    horizontal: T,
    vertical: T,
}

impl<T: Clone> Axes<T> {
    pub fn both(value: T) -> Self {
        Self::new(value.clone(), value)
    }

    pub fn horizontal(&self) -> T {
        self.horizontal.clone()
    }

    pub fn vertical(&self) -> T {
        self.vertical.clone()
    }

    pub fn set_horizontal(&mut self, value: T) {
        self.horizontal = value;
    }

    pub fn set_vertical(&mut self, value: T) {
        self.vertical = value;
    }
}

impl<T> Axes<T> {
    pub fn new(horizontal: T, vertical: T) -> Self {
        Self { horizontal, vertical }
    }
}

impl<T: Default> Default for Axes<T> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}