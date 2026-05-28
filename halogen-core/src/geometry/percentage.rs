use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Percentage(pub f32);

impl Percentage {
    pub const ZERO: Percentage = Percentage(0.0);

    pub const HALF: Percentage = Percentage(50.0);

    pub const FULL: Percentage = Percentage(100.0);
}

impl From<Percentage> for f32 {
    fn from(val: Percentage) -> Self {
        val.0
    }
}

impl From<f32> for Percentage {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl Add for Percentage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Percentage {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Percentage {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Percentage {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for Percentage {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<f32> for Percentage {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Mul<Self> for Percentage {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign<Self> for Percentage {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div<f32> for Percentage {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<f32> for Percentage {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Div<Self> for Percentage {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl DivAssign<Self> for Percentage {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl Rem<f32> for Percentage {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<f32> for Percentage {
    fn rem_assign(&mut self, rhs: f32) {
        self.0 %= rhs;
    }
}

impl Rem<Self> for Percentage {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}
impl RemAssign<Self> for Percentage {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Self::ZERO
    }
}