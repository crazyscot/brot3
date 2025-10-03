//! Arbitrary precision complex numbers, powered by `dashu::float::FBig`

use crate::big_vec2::BigVec2;
use dashu_float::FBig;
use std::ops::{Add, Deref, DerefMut, Sub};

/// Complex number using [`dashu::float::FBig`] as the underlying data type
///
/// ```
/// # use shader_util::big_complex::BigComplex;
/// # use shader_util::make_complex;
/// let x = make_complex!(1.0, 2.0);
/// let y = make_complex!(3.0, 4.0);
/// let z = x + y;
/// assert_eq!(z, make_complex!(4.0, 6.0));
/// let a = make_complex!(123, 456);
/// let b = a.clone() - a; // these are bignums, they do not support Copy
/// assert_eq!(b, BigComplex::ZERO);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct BigComplex(BigVec2);

/// Roughly creates a [`BigComplex`] from a pair of inputs
/// Intended for testing.
///
/// # Panics
/// If the numeric conversion failed
#[macro_export]
macro_rules! make_complex {
    ($x: expr, $y: expr) => {
        shader_util::big_complex::BigComplex::try_new($x, $y).unwrap()
    };
}

impl BigComplex {
    #[allow(missing_docs)]
    pub const ZERO: Self = Self(BigVec2::ZERO);

    /// Constructor
    #[must_use]
    pub const fn new(x: FBig, y: FBig) -> Self {
        Self(BigVec2::new(x, y))
    }

    /// Constructor from any type that can be converted to [`FBig`]
    ///
    /// ```
    /// # use shader_util::big_complex::BigComplex;
    /// let z = BigComplex::try_new(1.2, 3.4);
    /// ```
    pub fn try_new<T>(x: T, y: T) -> Result<Self, <FBig as TryFrom<T>>::Error>
    where
        FBig: std::convert::TryFrom<T>,
        //T: TryInto<FBig>,
    {
        let x = FBig::try_from(x)?;
        let y = FBig::try_from(y)?;
        Ok(Self::new(x, y))
    }

    /// Computes the square efficiently, consuming the original number.
    /// ```
    /// # use shader_util::make_complex;
    /// let x = make_complex!(0.0, 1.0);
    /// // i^2 = -1
    /// assert_eq!(x.square(), make_complex!(-1.0, 0.0));
    /// ```
    #[must_use]
    pub fn square(self) -> Self {
        const TWO: FBig = dashu::fbig!(10);
        Self::new(self.x.sqr() - self.y.sqr(), self.0.x * self.0.y * TWO)
    }

    /// Computes the square of the modulus of the complex.
    ///
    /// ```
    /// # use shader_util::make_complex;
    /// # use dashu::fbig;
    /// let x = make_complex!(0.0, 1.0);
    /// assert_eq!(x.norm_squared(), fbig!(1.0));
    /// ```
    #[must_use]
    pub fn norm_squared(&self) -> FBig {
        self.0.length_squared()
    }

    /// Accesses the precision
    #[must_use]
    pub fn precision(&self) -> glam::UVec2 {
        self.0.precision()
    }

    /// Sets the precision of both parts of the underlying data storage
    /// ```
    /// # use shader_util::big_complex::BigComplex;
    /// let z = BigComplex::ZERO.with_precision(123);
    /// let prec = z.precision();
    /// assert_eq!(prec.x, 123);
    /// assert_eq!(prec.y, 123);
    /// ```
    #[must_use]
    pub fn with_precision(self, precision: usize) -> Self {
        Self(self.0.with_precision(precision))
    }
}

impl Deref for BigComplex {
    type Target = BigVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BigComplex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BigVec2> for BigComplex {
    fn from(value: BigVec2) -> Self {
        BigComplex(value)
    }
}

impl Add for BigComplex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for BigComplex {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{BigComplex, BigVec2, FBig};
    use crate::make_bigvec2;
    use dashu_float::round::mode::Zero;
    #[test]
    fn conversions() {
        let v = make_bigvec2!(3, 4).with_precision(10);
        let v2 = make_bigvec2!(1, 1);
        let mut c = BigComplex::from(v);
        let r = &mut *c;
        assert_eq!(r.length_squared(), FBig::<Zero>::from(25));
        let z = r.clone() + v2;
        assert_eq!(z, make_bigvec2!(4, 5));
    }
}
