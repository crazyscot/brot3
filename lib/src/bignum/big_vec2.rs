//! Arbitrary precision version of [`Vec2`], powered by `dashu::float::FBig`

use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use dashu::float::{DBig, FBig, round::mode::Zero};
use easy_cast::ConvApprox as _;
use glam::{DVec2, UVec2, Vec2};
use serde::{Deserialize, Serialize};

/// Parse a decimal string into an arbitrary-precision [`FBig`].
/// Preserves all significant digits in the string.
///
/// # Panics
/// If the string is not a valid decimal number.
#[must_use]
pub fn fbig_from_str(s: &str) -> FBig {
    DBig::from_str(s)
        .expect("failed to parse decimal string")
        .with_base::<2>()
        .value()
        .with_rounding::<Zero>()
}

/// Serde helper: serialises [`FBig`] as `(significand, exponent)` where `significand` is a
/// decimal integer string and `exponent` is the binary exponent (`value = sig * 2^exp`).
/// This is more compact than dashu's default binary-string representation.
mod fbig_serde {
    use std::str::FromStr;

    use dashu::{float::FBig, integer::IBig};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S: Serializer>(value: &FBig, serializer: S) -> Result<S::Ok, S::Error> {
        let repr = value.repr();
        (repr.significand().to_string(), repr.exponent()).serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<FBig, D::Error> {
        let (sig_str, exp): (String, isize) = Deserialize::deserialize(deserializer)?;
        let sig = IBig::from_str(&sig_str).map_err(serde::de::Error::custom)?;
        Ok(FBig::from_parts(sig, exp))
    }
}

/// Arbitrary precision version of [`glam::Vec2`]
///
/// Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct BigVec2 {
    #[serde(with = "fbig_serde")]
    pub x: FBig,
    #[serde(with = "fbig_serde")]
    pub y: FBig,
}

/// Roughly creates a [`BigVec2`] from a pair of inputs.
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

/// Creates a [`BigVec2`] from a pair of decimal strings with full precision.
/// Intended for testing.
///
/// ```
/// # use brot3_lib::make_bigvec2_str;
/// let v = make_bigvec2_str!("1.25", "-3.5");
/// assert_eq!(v.x.to_f64().value(), 1.25);
/// assert_eq!(v.y.to_f64().value(), -3.5);
/// ```
///
/// # Panics
/// If parsing fails or numeric conversion failed
#[macro_export]
macro_rules! make_bigvec2_str {
    ($x: expr, $y: expr) => {{
        let x = $crate::fbig_from_str($x);
        let y = $crate::fbig_from_str($y);
        $crate::BigVec2::new(x, y)
    }};
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
    /// # use brot3_lib::BigVec2;
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
        glam::uvec2(u32::conv_approx(p0), u32::conv_approx(p1))
    }

    /// Returns the greater precision of either axis
    #[must_use]
    pub fn precision_larger(&self) -> usize {
        self.x.precision().max(self.y.precision())
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

impl AsRef<BigVec2> for BigVec2 {
    fn as_ref(&self) -> &BigVec2 {
        self
    }
}

impl<V: AsRef<BigVec2>> Add<V> for BigVec2 {
    type Output = Self;

    fn add(self, other: V) -> Self::Output {
        Self::new(self.x + &other.as_ref().x, self.y + &other.as_ref().y)
    }
}

impl<V: AsRef<BigVec2>> Sub<V> for BigVec2 {
    type Output = Self;

    fn sub(self, other: V) -> Self::Output {
        Self::new(self.x - &other.as_ref().x, self.y - &other.as_ref().y)
    }
}

impl<V: AsRef<BigVec2>> AddAssign<V> for BigVec2 {
    fn add_assign(&mut self, other: V) {
        self.x += &other.as_ref().x;
        self.y += &other.as_ref().y;
    }
}
impl<V: AsRef<BigVec2>> SubAssign<V> for BigVec2 {
    fn sub_assign(&mut self, other: V) {
        self.x -= &other.as_ref().x;
        self.y -= &other.as_ref().y;
    }
}

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
    fn div_assign(&mut self, rhs: f64) {
        let factor = FBig::try_from(rhs).unwrap();
        self.x /= &factor;
        self.y /= factor;
    }
}

impl Div<f64> for BigVec2 {
    type Output = Self;

    fn div(mut self, other: f64) -> Self::Output {
        self /= other;
        self
    }
}

impl Mul<f64> for BigVec2 {
    type Output = Self;

    fn mul(mut self, other: f64) -> Self::Output {
        self *= other;
        self
    }
}

impl MulAssign<f64> for BigVec2 {
    fn mul_assign(&mut self, rhs: f64) {
        let factor = FBig::try_from(rhs).unwrap();
        self.x *= &factor;
        self.y *= factor;
    }
}

impl std::fmt::Display for BigVec2 {
    /// Converts to a string representation (binary)
    /// ```
    /// # use brot3_lib::{BigVec2, make_bigvec2};
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
    use dashu_float::FBig;
    use glam::dvec2;

    use super::{BigVec2, DVec2};
    use crate::make_bigvec2;
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
        v2 -= &make_bigvec2!(3., 5.);
        assert_eq!(v2, BigVec2::ZERO);

        let mut a1 = make_bigvec2!(20., 12.);
        let dv1 = dvec2(4., 4.);
        let a2 = a1.clone() - BigVec2::try_from(dv1).unwrap();
        a1 -= dv1;
        assert_eq!(a1, a2);
        a1 /= 2.0;
        assert_eq!(a1, make_bigvec2!(8., 4.));

        assert_eq!(a1.precision(), glam::uvec2(53, 53));

        let dv2 = dvec2(0., 6.);
        a1 += dv2;
        assert_eq!(a1.to_string(), "BigVec2(1000, 1010)");
        let mut a2 = a1 / 2.;
        assert_eq!(a2, make_bigvec2!(4., 5.));

        a2 *= 3.0;
        assert_eq!(a2, make_bigvec2!(12., 15.));
        let a3 = a2 * 2.0;
        assert_eq!(a3, make_bigvec2!(24., 30.));
    }

    #[test]
    fn precision_larger() {
        let x = FBig::from(42).with_precision(128).value();
        let y = FBig::from(42).with_precision(192).value();
        let z = BigVec2::new(x, y);
        assert_eq!(z.precision_larger(), 192);
    }

    #[test]
    fn serde_roundtrip() {
        let original = make_bigvec2!(1.25, -3.5);
        let json = serde_json::to_string(&original).expect("serialization failed");
        println!("JSON: {json}");
        let restored: BigVec2 = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(original, restored);
    }

    #[test]
    fn serde_roundtrip_zero() {
        let original = BigVec2::ZERO;
        let json = serde_json::to_string(&original).expect("serialization failed");
        println!("JSON: {json}");
        let restored: BigVec2 = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(original, restored);
    }

    #[test]
    fn serde_roundtrip_high_precision() {
        let x = FBig::from(42).with_precision(128).value();
        let y = FBig::from(-7).with_precision(192).value();
        let original = BigVec2::new(x, y);
        let json = serde_json::to_string(&original).expect("serialization failed");
        println!("JSON: {json}");
        let restored: BigVec2 = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(original, restored);
    }
}
