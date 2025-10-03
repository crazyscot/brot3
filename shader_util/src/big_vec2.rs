//! Arbitrary precision version of [`Vec2`], powered by `dashu::float::FBig`

use dashu::float::FBig;
use glam::{DVec2, UVec2, Vec2};
use std::ops::{Add, AddAssign, Div, DivAssign, Sub, SubAssign};

/// Arbitrary precision version of [`glam::Vec2`]
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub struct BigVec2 {
    pub x: FBig,
    pub y: FBig,
}

/// Roughly creates a [`BigVec2`] from a pair of inputs
/// Intended for testing.
///
/// # Panics
/// If the numeric conversion failed
#[macro_export]
macro_rules! make_bigvec2 {
    ($x: expr, $y: expr) => {
        BigVec2::try_new($x, $y).unwrap()
    };
}
impl BigVec2 {
    #[allow(missing_docs)]
    pub const ZERO: Self = Self::new(FBig::ZERO, FBig::ZERO);

    /// Constructor
    #[must_use]
    pub const fn new(x: FBig, y: FBig) -> Self {
        Self { x, y }
    }

    /// Constructor from any type that can be converted to [`FBig`]
    ///
    /// ```
    /// # use shader_util::big_vec2::BigVec2;
    /// let z = BigVec2::try_new(1.2, 3.4);
    /// ```
    pub fn try_new<T>(x: T, y: T) -> Result<Self, <FBig as TryFrom<T>>::Error>
    where
        FBig: std::convert::TryFrom<T>,
    {
        let x = FBig::try_from(x)?;
        let y = FBig::try_from(y)?;
        Ok(Self::new(x, y))
    }

    /// Returns the current precision of the pair of axes
    #[must_use]
    pub fn precision(&self) -> UVec2 {
        let p0 = self.x.precision();
        let p1 = self.y.precision();
        #[allow(clippy::cast_possible_truncation)]
        glam::uvec2(p0 as u32, p1 as u32)
    }

    /// Sets the precision
    #[must_use]
    pub fn with_precision(mut self, precision: usize) -> Self {
        self.x = self.x.with_precision(precision).value();
        self.y = self.y.with_precision(precision).value();
        self
    }

    /// Converts to a [`Vec2`], possibly losing precision
    #[must_use]
    pub fn as_vec2(&self) -> Vec2 {
        glam::vec2(self.x.to_f32().value(), self.y.to_f32().value())
    }

    /// Converts to a [`DVec2`], possibly losing precision
    #[must_use]
    pub fn as_dvec2(&self) -> DVec2 {
        glam::dvec2(self.x.to_f64().value(), self.y.to_f64().value())
    }

    /// Computes `x^2 + y^2`
    #[must_use]
    pub fn length_squared(&self) -> FBig {
        self.x.sqr() + self.y.sqr()
    }
}

impl TryFrom<glam::DVec2> for BigVec2 {
    type Error = dashu::base::ConversionError;

    fn try_from(v: glam::DVec2) -> Result<Self, Self::Error> {
        let x = FBig::try_from(v.x)?;
        let y = FBig::try_from(v.y)?;
        Ok(Self { x, y })
    }
}

impl Add for BigVec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}
impl Sub for BigVec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl AddAssign for BigVec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl SubAssign for BigVec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

// impl AddAssign<glam::Vec2> for Vec2Big {
//     fn add_assign(&mut self, other: glam::Vec2) {
//         self.x += FBig::try_from(other.x).unwrap();
//         self.y += FBig::try_from(other.y).unwrap();
//     }
// }

// impl SubAssign<glam::Vec2> for Vec2Big {
//     fn sub_assign(&mut self, other: glam::Vec2) {
//         self.x -= FBig::try_from(other.x).unwrap();
//         self.y -= FBig::try_from(other.y).unwrap();
//     }
// }

impl AddAssign<glam::DVec2> for BigVec2 {
    fn add_assign(&mut self, other: glam::DVec2) {
        self.x += FBig::try_from(other.x).unwrap();
        self.y += FBig::try_from(other.y).unwrap();
    }
}

impl SubAssign<glam::DVec2> for BigVec2 {
    fn sub_assign(&mut self, other: glam::DVec2) {
        self.x -= FBig::try_from(other.x).unwrap();
        self.y -= FBig::try_from(other.y).unwrap();
    }
}

impl DivAssign<f64> for BigVec2 {
    fn div_assign(&mut self, other: f64) {
        self.x /= FBig::try_from(other).unwrap();
        self.y /= FBig::try_from(other).unwrap();
    }
}

impl Div<f64> for BigVec2 {
    type Output = Self;
    fn div(mut self, other: f64) -> Self::Output {
        self /= other;
        self
    }
}

impl std::fmt::Display for BigVec2 {
    /// Converts to a string representation (binary)
    /// ```
    /// # use shader_util::big_vec2::BigVec2;
    /// # use shader_util::make_bigvec2;
    /// let v = make_bigvec2!(15., 2.);
    /// assert_eq!(v.to_string(), "BigVec2(1111, 10)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BigVec2(")?;
        self.x.fmt(f)?;
        write!(f, ", ")?;
        self.y.fmt(f)?;
        write!(f, ")")
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{BigVec2, DVec2};
    use crate::make_bigvec2;
    use glam::dvec2;
    #[test]
    fn conversions() {
        let z = make_bigvec2!(3, 4).with_precision(10);
        let vv = z.as_vec2();
        let dv = z.as_dvec2();
        assert_eq!(DVec2::from(vv), dv);
        let z2 = BigVec2::try_from(dv).unwrap();
        assert_eq!(z, z2);
    }

    #[test]
    fn arithmetic() {
        let v1 = make_bigvec2!(0., 1.);
        let mut v2 = make_bigvec2!(3., 4.);
        v2 += v1;
        assert_eq!(v2, make_bigvec2!(3., 5.));
        v2 -= make_bigvec2!(3., 5.);
        assert_eq!(v2, BigVec2::ZERO);

        let mut a1 = make_bigvec2!(20., 12.);
        let dv1 = dvec2(4., 4.);
        a1 -= dv1;
        a1 /= 2.0;
        assert_eq!(a1, make_bigvec2!(8., 4.));

        let dv2 = dvec2(0., 6.);
        a1 += dv2;
        assert_eq!(a1.to_string(), "BigVec2(1000, 1010)");
        let a2 = a1 / 2.;
        assert_eq!(a2, make_bigvec2!(4., 5.));
    }
}
