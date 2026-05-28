use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct ScaleUnits(pub f32);

impl ScaleUnits {
    pub const ZERO: ScaleUnits = ScaleUnits(0.0);
}

impl From<ScaleUnits> for f32 {
    fn from(val: ScaleUnits) -> Self {
        val.0
    }
}

impl From<f32> for ScaleUnits {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl Add for ScaleUnits {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for ScaleUnits {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for ScaleUnits {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for ScaleUnits {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for ScaleUnits {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<f32> for ScaleUnits {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Mul<Self> for ScaleUnits {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign<Self> for ScaleUnits {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div<f32> for ScaleUnits {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<f32> for ScaleUnits {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Div<Self> for ScaleUnits {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl DivAssign<Self> for ScaleUnits {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl Rem<f32> for ScaleUnits {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<f32> for ScaleUnits {
    fn rem_assign(&mut self, rhs: f32) {
        self.0 %= rhs;
    }
}

impl Rem<Self> for ScaleUnits {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}
impl RemAssign<Self> for ScaleUnits {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Default for ScaleUnits {
    fn default() -> Self {
        Self::ZERO
    }
}