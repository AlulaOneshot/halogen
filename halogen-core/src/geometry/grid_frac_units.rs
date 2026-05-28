use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct GridFractionalUnits(pub f32);

impl GridFractionalUnits {
    pub const ZERO: GridFractionalUnits = GridFractionalUnits(0.0);
}

impl From<GridFractionalUnits> for f32 {
    fn from(val: GridFractionalUnits) -> Self {
        val.0
    }
}

impl From<f32> for GridFractionalUnits {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl Add for GridFractionalUnits {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for GridFractionalUnits {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for GridFractionalUnits {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for GridFractionalUnits {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for GridFractionalUnits {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<f32> for GridFractionalUnits {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Mul<Self> for GridFractionalUnits {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign<Self> for GridFractionalUnits {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div<f32> for GridFractionalUnits {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<f32> for GridFractionalUnits {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Div<Self> for GridFractionalUnits {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl DivAssign<Self> for GridFractionalUnits {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl Rem<f32> for GridFractionalUnits {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<f32> for GridFractionalUnits {
    fn rem_assign(&mut self, rhs: f32) {
        self.0 %= rhs;
    }
}

impl Rem<Self> for GridFractionalUnits {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}
impl RemAssign<Self> for GridFractionalUnits {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Default for GridFractionalUnits {
    fn default() -> Self {
        Self::ZERO
    }
}