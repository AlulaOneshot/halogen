use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Pixels(pub f32);

impl Pixels {
    pub const ZERO: Pixels = Pixels(0.0);
}

impl From<Pixels> for f32 {
    fn from(val: Pixels) -> Self {
        val.0
    }
}

impl From<f32> for Pixels {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl Add for Pixels {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Pixels {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Pixels {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Pixels {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for Pixels {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<f32> for Pixels {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Mul<Self> for Pixels {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign<Self> for Pixels {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div<f32> for Pixels {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<f32> for Pixels {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Div<Self> for Pixels {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl DivAssign<Self> for Pixels {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl Rem<f32> for Pixels {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<f32> for Pixels {
    fn rem_assign(&mut self, rhs: f32) {
        self.0 %= rhs;
    }
}

impl Rem<Self> for Pixels {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}
impl RemAssign<Self> for Pixels {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Default for Pixels {
    fn default() -> Self {
        Self::ZERO
    }
}