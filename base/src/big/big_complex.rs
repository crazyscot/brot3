//! Arbitrary precision complex numbers, powered by `dashu::float::FBig`

use std::ops::{Add, Deref, DerefMut, Div, Sub};

use dashu_float::FBig;
use serde::{Deserialize, Serialize};

use crate::BigVec2;

/// Arbitrary precision complex number using `dashu_float::FBig` as the underlying data type
///
///
/// ```
/// # use base::{BigComplex, make_bigcomplex};
/// let x = make_bigcomplex!(1.0, 2.0);
/// let y = make_bigcomplex!(3.0, 4.0);
/// let z = x + y;
/// assert_eq!(z, make_bigcomplex!(4.0, 6.0));
/// let a = make_bigcomplex!(123, 456);
/// let b = a.clone() - a; // these are bignums, they do not support Copy
/// assert_eq!(b, BigComplex::ZERO);
/// ```
#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]

pub struct BigComplex(pub BigVec2);

/// Roughly creates a [`BigComplex`] from a pair of inputs.
/// Intended for testing.
///
/// # Panics
/// If the numeric conversion failed
#[macro_export]
macro_rules! make_bigcomplex {
    ($x: expr, $y: expr) => {
        $crate::BigComplex::try_new($x, $y).unwrap()
    };
}

/// Creates a [`BigComplex`] from a pair of decimal strings with full precision.
/// Intended for testing.
///
/// ```
/// # use base::make_bigcomplex_str;
/// let z = make_bigcomplex_str!("1.25", "-3.5");
/// assert_eq!(z.x.to_f64().value(), 1.25);
/// assert_eq!(z.y.to_f64().value(), -3.5);
/// ```
///
/// # Panics
/// If parsing fails or numeric conversion failed
#[macro_export]
macro_rules! make_bigcomplex_str {
    ($x: expr, $y: expr) => {{
        let x = $crate::fbig_from_str($x);
        let y = $crate::fbig_from_str($y);
        $crate::BigComplex::new(x, y)
    }};
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
    /// # use base::BigComplex;
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
    /// # use base::make_bigcomplex;
    /// let x = make_bigcomplex!(0.0, 1.0);
    /// // i^2 = -1
    /// assert_eq!(x.square(), make_bigcomplex!(-1.0, 0.0));
    /// ```
    #[must_use]
    pub fn square(self) -> Self {
        const TWO: FBig = dashu::fbig!(10);
        Self::new(self.x.sqr() - self.y.sqr(), self.0.x * self.0.y * TWO)
    }

    /// Computes the square of the modulus of the complex.
    ///
    /// ```
    /// # use base::make_bigcomplex;
    /// # use dashu::{fbig, float::FBig, float::round};
    /// let x = make_bigcomplex!(0.0, 1.0);
    /// assert_eq!(x.norm_squared(), fbig!(1.0));
    /// let z = make_bigcomplex!(5.0, 4.0);
    /// assert_eq!(z.norm_squared(), FBig::<round::mode::Zero>::from(41));
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
    /// # use base::BigComplex;
    /// let z = BigComplex::ZERO.with_precision(123);
    /// let prec = z.precision();
    /// assert_eq!(prec.x, 123);
    /// assert_eq!(prec.y, 123);
    /// ```
    #[must_use]
    pub fn with_precision(self, precision: usize) -> Self {
        Self(self.0.with_precision(precision))
    }

    #[must_use]
    /// Computes the complex conjugate
    /// ```
    /// # use base::make_bigcomplex;
    /// let x = make_bigcomplex!(0.0, 1.0);
    /// assert_eq!(x.conjugate(), make_bigcomplex!(0.0, -1.0));
    /// let z = make_bigcomplex!(12.0, 34.0);
    /// assert_eq!(z.conjugate(), make_bigcomplex!(12.0, -34.0));
    /// ```
    pub fn conjugate(mut self) -> Self {
        self.y *= dashu::base::Sign::Negative;
        self
    }

    /// Computes the reciprocal
    /// ```
    /// # use base::make_bigcomplex;
    /// let z = make_bigcomplex!(2.0, 0.0);
    /// assert_eq!(z.recip(), make_bigcomplex!(0.5, 0.0));
    /// let z = make_bigcomplex!(0.0, 1.0);
    /// assert_eq!(z.recip(), make_bigcomplex!(0.0, -1.0));
    /// let z = make_bigcomplex!(0.4, -0.2);
    /// let recip = z.clone().recip();
    /// assert_eq!(recip.x.to_f64().value(), 2.0);
    /// assert_eq!(recip.y.to_f64().value(), 1.0);
    /// ```
    #[must_use]
    pub fn recip(self) -> Self {
        let nsq = self.norm_squared();
        self.conjugate() / &nsq
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

impl std::fmt::Debug for BigComplex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for BigComplex {
    /// Displays the complex number in `a + bi` notation using decimal representation.
    ///
    /// ```
    /// # use base::make_bigcomplex;
    /// let z = make_bigcomplex!(1.25, -3.5);
    /// assert_eq!(z.to_string(), "1.25 - 3.5i");
    /// let z2 = make_bigcomplex!(1.25, 3.5);
    /// assert_eq!(z2.to_string(), "1.25 + 3.5i");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use dashu::base::Sign;
        let re = self.x.to_decimal().value();
        let im = self.y.to_decimal().value();
        match self.y.sign() {
            Sign::Negative => write!(f, "{re} - {}i", -im),
            Sign::Positive => write!(f, "{re} + {im}i"),
        }
    }
}

impl Add for BigComplex {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + &other.0)
    }
}
impl Add<&BigComplex> for BigComplex {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        Self(self.0 + &other.0)
    }
}

impl Sub for BigComplex {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - &other.0)
    }
}
impl Sub<&BigComplex> for BigComplex {
    type Output = Self;

    fn sub(self, other: &Self) -> Self::Output {
        Self(self.0 - &other.0)
    }
}

impl Div<&FBig> for BigComplex {
    type Output = Self;

    fn div(self, rhs: &FBig) -> Self::Output {
        let (x, y) = (self.0.x, self.0.y);
        Self(BigVec2 {
            x: x / rhs,
            y: y / rhs,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use dashu::fbig;
    use dashu_float::round::mode::Zero;

    use super::{BigVec2, FBig};
    use crate::{BigComplex, make_bigcomplex, make_bigvec2};

    #[test]
    fn conversions() {
        let v = make_bigvec2!(3, 4).with_precision(10);
        let v2 = make_bigvec2!(1, 1);
        let mut c = BigComplex::from(v);
        let r = &mut *c;
        assert_eq!(r.length_squared(), FBig::<Zero>::from(25));
        let z = r.clone() + &v2;
        assert_eq!(z, make_bigvec2!(4, 5));
    }

    #[test]
    fn exercise() {
        let c1 = make_bigcomplex!(1.0, 2.0);
        let c2 = BigComplex::try_new(3.0, 4.0).unwrap();
        let z = c1 + &c2;
        let z = z + c2;
        assert_eq!(z, make_bigcomplex!(7.0, 10.0));
        let a = make_bigcomplex!(123, 456);
        let b = a.clone() - a; // these are bignums, they do not support Copy
        assert_eq!(b, BigComplex::ZERO);
        assert_eq!(z.clone() - &z, BigComplex::ZERO);
    }

    #[test]
    fn square() {
        let x = make_bigcomplex!(0.0, 1.0);
        // i^2 = -1
        assert_eq!(x.square(), make_bigcomplex!(-1.0, 0.0));
    }

    #[test]
    fn mod_squared() {
        let x = make_bigcomplex!(0.0, 1.0);
        assert_eq!(x.norm_squared(), fbig!(1.0));
    }

    #[test]
    fn precision() {
        let z = BigComplex::ZERO.with_precision(123);
        let prec = z.precision();
        assert_eq!(prec.x, 123);
        assert_eq!(prec.y, 123);
    }

    #[test]
    fn conjugate() {
        let z = make_bigcomplex!(1.0, 2.0);
        let expected = make_bigcomplex!(1.0, -2.0);
        assert_eq!(z.conjugate(), expected);
    }
    #[test]
    fn reciprocal() {
        let z = make_bigcomplex!(2.0, 2.0);
        let expected = make_bigcomplex!(0.25, -0.25);
        assert_eq!(z.recip(), expected);
    }

    #[test]
    fn serialise() {
        let z = make_bigcomplex_str!("-1.378186747593672212", "-0.0177134138869923");
        let json = serde_json::to_string(&z).expect("serialization failed");
        println!("z: {z}");
        println!("JSON: {json}");
        let z2: BigComplex = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(z, z2);
    }
}
