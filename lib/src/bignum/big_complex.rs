//! Arbitrary precision complex numbers, powered by `dashu::float::FBig`

use std::{
    ops::{Add, Deref, DerefMut, Div, Sub},
    str::FromStr,
};

use dashu_float::FBig;
use easy_cast::{Cast as _, Conv as _, ConvFloat as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BigVec2, fbig_from_str};

/// Arbitrary precision complex number using `dashu_float::FBig` as the underlying data type
///
/// Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>
///
/// ```
/// # use brot3_lib::{BigComplex, make_bigcomplex};
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
/// # use brot3_lib::make_bigcomplex_str;
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

/// Error type for parsing [`BigComplex`] from strings.
///
/// This enum provides detailed context for why a parse operation failed,
/// making it easier to diagnose input errors.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ParseError {
    /// The input format is invalid (e.g., missing imaginary unit 'i').
    #[error("Invalid complex number format: {0}")]
    InvalidFormat(String),
    /// The real component failed to parse.
    #[error("Invalid real part: {0}")]
    InvalidRealPart(String),
    /// The imaginary component failed to parse.
    #[error("Invalid imaginary part: {0}")]
    InvalidImaginaryPart(String),
    /// Neither real nor imaginary part could be identified.
    #[error("Missing complex number: neither real nor imaginary part found")]
    MissingComplex,
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
    /// # use brot3_lib::BigComplex;
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
    /// # use brot3_lib::make_bigcomplex;
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
    /// # use brot3_lib::make_bigcomplex;
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
    /// # use brot3_lib::BigComplex;
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
    /// # use brot3_lib::make_bigcomplex;
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
    /// # use brot3_lib::make_bigcomplex;
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

/// Parse a complex number string into its real and imaginary components.
///
/// Supports flexible formats:
/// - Bare real: "1.5", "-2.3"
/// - Bare imaginary: "2.3i", "-2.3i", "i", "-i"
/// - Full complex: "1.5 + 2.3i", "1.5+2.3i", "1.5 - 2.3i", "-1.5-2.3i"
/// - With optional spaces around operators
///
/// Returns `(real_str, imag_str, precision)` where:
/// - `real_str` is the parseable real component (may be empty for imaginary-only)
/// - `imag_str` is the parseable imaginary component (may be empty for real-only)
/// - `precision` is estimated significant digits (128 bits if both are zero)
///
/// # Errors
/// Returns `ParseError` if the input is malformed.
fn parse_bigcomplex_input(s: &str) -> Result<(String, String, usize), ParseError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ParseError::MissingComplex);
    }

    // Check if there's an 'i' (imaginary unit) in the string
    let has_imaginary = trimmed.contains('i') || trimmed.contains('I');

    // If no 'i', it's a bare real number
    if !has_imaginary {
        if !is_valid_decimal(trimmed) {
            return Err(ParseError::InvalidRealPart(trimmed.to_string()));
        }
        let precision = estimate_precision(trimmed, "");
        return Ok((trimmed.to_string(), String::new(), precision));
    }

    // If there's exactly one number (before 'i'), it might be bare imaginary
    // Look for +/- that separates real and imaginary parts

    // Find the last +/- operator that's not at the start
    let mut last_op_index = None;

    for (i, ch) in trimmed.chars().enumerate() {
        if i > 0 && (ch == '+' || ch == '-') {
            last_op_index = Some(i);
        }
    }

    let (real_part, imag_part) = if let Some(op_idx) = last_op_index {
        // Split at the operator
        let (before, after) = trimmed.split_at(op_idx);
        let operator = &trimmed[op_idx..=op_idx];

        // before should be the real part (possibly empty or with leading sign)
        let before_trimmed = before.trim();
        let after_trimmed = after[1..].trim(); // skip the operator itself

        if before_trimmed.is_empty() {
            // No real part, treat as bare imaginary
            (String::new(), format!("{operator}{after_trimmed}"))
        } else {
            (
                before_trimmed.to_string(),
                format!("{operator}{after_trimmed}"),
            )
        }
    } else {
        // No +/- operator found, must be bare imaginary (just "2.3i" or "i")
        (String::new(), trimmed.to_string())
    };

    // Normalize imaginary part: remove 'i' or 'I' suffix
    let imag_cleaned = imag_part.trim_end_matches('i').trim_end_matches('I');
    let mut imag_part = imag_cleaned.trim().to_string();

    // Handle bare "i" or "-i" case
    if imag_part.is_empty() || imag_part == "+" {
        imag_part = "1".to_string();
    } else if imag_part == "-" {
        imag_part = "-1".to_string();
    }

    // Validate that both parts are valid decimal strings or empty
    if !real_part.is_empty() && !is_valid_decimal(&real_part) {
        return Err(ParseError::InvalidRealPart(real_part));
    }
    if !imag_part.is_empty() && !is_valid_decimal(&imag_part) {
        return Err(ParseError::InvalidImaginaryPart(imag_part));
    }

    if real_part.is_empty() && imag_part.is_empty() {
        return Err(ParseError::MissingComplex);
    }

    let precision = estimate_precision(&real_part, &imag_part);
    Ok((real_part, imag_part, precision))
}

/// Check if a string is a valid decimal number format.
fn is_valid_decimal(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let mut chars = trimmed.chars().peekable();

    // Optional leading sign
    if let Some(&ch) = chars.peek()
        && (ch == '+' || ch == '-')
    {
        let _ = chars.next();
    }

    // At least one digit required
    if !chars.peek().is_some_and(char::is_ascii_digit) {
        return false;
    }

    let mut seen_dot = false;
    let mut seen_e = false;

    while let Some(ch) = chars.next() {
        match ch {
            '0'..='9' => {}
            '.' if !seen_dot && !seen_e => seen_dot = true,
            'e' | 'E' if !seen_e => {
                seen_e = true;
                // After 'e', optional sign then at least one digit
                if let Some(&next) = chars.peek()
                    && (next == '+' || next == '-')
                {
                    let _ = chars.next();
                }
                if !chars.peek().is_some_and(char::is_ascii_digit) {
                    return false;
                }
            }
            _ => return false,
        }
    }

    true
}

/// Count digits, ignoring leading zeros and the decimal point.
/// Trailing zeros are included.
fn count_significant_digits(s: &str) -> usize {
    let trimmed = s.trim().trim_start_matches(['+', '-']);
    trimmed
        .chars()
        .skip_while(|c| *c == '0' || *c == '.')
        .filter(char::is_ascii_digit)
        .count()
}

/// Estimate the precision (in bits) needed to represent either part of a complex number.
/// This assumes the two parts have similar precision requirements.
fn estimate_precision(real: &str, imag: &str) -> usize {
    let digits = count_significant_digits(real)
        .max(count_significant_digits(imag))
        .max(1);
    // Rough heuristic: ~3.3 bits per decimal digit
    u64::conv_trunc((f64::conv(digits) * 3.3).ceil()).cast()
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
    /// # use brot3_lib::make_bigcomplex;
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

impl FromStr for BigComplex {
    type Err = ParseError;

    /// Parse a string into a [`BigComplex`] number.
    ///
    /// Supports flexible formats with optional whitespace:
    /// - Bare real numbers: `"1.5"`, `"-2.3"`
    /// - Bare imaginary numbers: `"2.3i"`, `"-2.3i"`, `"i"`, `"-i"`
    /// - Full complex notation: `"1.5 + 2.3i"`, `"1.5+2.3i"`, `"1.5 - 2.3i"`
    ///
    /// Precision is automatically determined from the number of significant digits in the input.
    /// Scientific notation is supported (inherited from the underlying parser).
    ///
    /// # Examples
    /// ```
    /// # use std::str::FromStr;
    /// # use brot3_lib::BigComplex;
    /// # use float_eq::assert_float_eq;
    /// let z: BigComplex = "1.5 + 2.3i".parse().unwrap();
    /// assert_float_eq!(z.x.to_decimal().value().to_f64().value(), 1.5, ulps <= 4);
    /// assert_float_eq!(z.y.to_decimal().value().to_f64().value(), 2.3, ulps <= 4);
    ///
    /// let bare_real: BigComplex = "3.14".parse().unwrap();
    /// assert_float_eq!(
    ///     bare_real.y.to_decimal().value().to_f64().value(),
    ///     0.0,
    ///     abs <= 1e-10
    /// );
    ///
    /// let bare_imag: BigComplex = "2.5i".parse().unwrap();
    /// assert_float_eq!(
    ///     bare_imag.x.to_decimal().value().to_f64().value(),
    ///     0.0,
    ///     abs <= 1e-10
    /// );
    /// assert_float_eq!(
    ///     bare_imag.y.to_decimal().value().to_f64().value(),
    ///     2.5,
    ///     ulps <= 4
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (real_str, imag_str, precision) = parse_bigcomplex_input(s)?;

        // Parse real component
        let x = if real_str.is_empty() {
            FBig::ZERO
        } else {
            fbig_from_str(&real_str)
        };

        // Parse imaginary component
        let y = if imag_str.is_empty() {
            FBig::ZERO
        } else {
            fbig_from_str(&imag_str)
        };

        // Some numbers require infinite precision, so we estimate _desired_ precision from the
        // input.
        Ok(BigComplex::new(x, y).with_precision(precision))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use dashu::fbig;
    use dashu_float::round::mode::Zero;
    use float_eq::assert_float_eq;

    use super::{BigVec2, FBig, ParseError};
    use crate::{BigComplex, make_bigvec2};

    /// Helper trait for converting a float that came from decimal out to an f64 using the correct
    /// precision and rounding
    trait FBigExt {
        fn f64_decimal(&self) -> f64;
    }
    impl FBigExt for FBig {
        fn f64_decimal(&self) -> f64 {
            self.to_decimal().value().to_f64().value()
        }
    }

    macro_rules! afnear {
        ($a:expr, $b:expr) => {
            assert_float_eq!($a, $b, ulps <= 4, abs <= 1e-10);
        };
    }

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

    #[test]
    fn from_str_full_complex() {
        // Standard format with spaces
        let z: BigComplex = "1.5 + 2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);

        // Standard format without spaces
        let z: BigComplex = "1.5+2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);

        // Negative imaginary
        let z: BigComplex = "1.5 - 2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), -2.3);

        // Both negative
        let z: BigComplex = "-1.5-2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), -1.5);
        afnear!(z.y.f64_decimal(), -2.3);

        // Leading positive sign
        let z: BigComplex = "+1.5+2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);
    }

    #[test]
    fn from_str_bare_real() {
        // Positive real
        let z: BigComplex = "1.5".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 0.0);

        // Negative real
        let z: BigComplex = "-2.3".parse().unwrap();
        afnear!(z.x.f64_decimal(), -2.3);
        afnear!(z.y.f64_decimal(), 0.0);

        // Integer real
        let z: BigComplex = "42".parse().unwrap();
        afnear!(z.x.f64_decimal(), 42.0);
        afnear!(z.y.f64_decimal(), 0.0);
    }

    #[test]
    fn from_str_bare_imaginary() {
        // Simple imaginary
        let z: BigComplex = "2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), 2.3);

        // Negative imaginary
        let z: BigComplex = "-2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), -2.3);

        // Unit imaginary (implicit 1)
        let z: BigComplex = "i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), 1.0);

        // Negative unit imaginary
        let z: BigComplex = "-i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), -1.0);
    }

    #[test]
    fn from_str_edge_cases() {
        // Zero
        let z: BigComplex = "0".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), 0.0);

        // Zero + zero i
        let z: BigComplex = "0+0i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), 0.0);

        // With extra spaces
        let z: BigComplex = "1.5  +  2.3i".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);

        // Leading/trailing spaces
        let z: BigComplex = "  1.5 + 2.3i  ".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);
    }

    #[test]
    fn from_str_roundtrip_display() {
        let original_strs = vec!["1.5+2.3i", "1.5-2.3i", "-1.5-2.3i"];

        for original_str in original_strs {
            let z: BigComplex = original_str.parse().unwrap();
            let displayed = z.to_string();
            let z2: BigComplex = displayed.parse().unwrap();
            // Check that values match (with reasonable precision from binary representation)
            afnear!(z.x.f64_decimal(), z2.x.f64_decimal());
            afnear!(z.y.f64_decimal(), z2.y.f64_decimal());
        }
    }

    #[test]
    fn from_str_precision_preservation() {
        // Long decimal - should preserve significant digits
        let z: BigComplex = "1.123456789012345678901234567890+2.987654321098765432109876543210i"
            .parse()
            .unwrap();

        // Check that precision was set (not exact equality due to rounding, but close)
        let prec = z.precision();
        assert!(
            prec.x > 50,
            "Real precision should be > 50 bits for 30 decimal digits"
        );
        assert!(
            prec.y > 50,
            "Imaginary precision should be > 50 bits for 30 decimal digits"
        );
    }

    #[test]
    fn from_str_errors() {
        // These should all return parse errors, not panics

        // Empty string
        let result: Result<BigComplex, _> = "".parse();
        assert!(matches!(result, Err(ParseError::MissingComplex)));

        // Only operators/signs
        let result: Result<BigComplex, _> = "+-".parse();
        assert!(result.is_err());

        // Invalid real part with double dot
        let result: Result<BigComplex, _> = "1.5.5+2.3i".parse();
        assert!(result.is_err());

        // Invalid imaginary part with double dot
        let result: Result<BigComplex, _> = "1.5+2.5.3i".parse();
        assert!(result.is_err());
    }

    #[test]
    fn from_str_uppercase_i() {
        // Should work with uppercase I as well
        let z: BigComplex = "2.3I".parse().unwrap();
        afnear!(z.x.f64_decimal(), 0.0);
        afnear!(z.y.f64_decimal(), 2.3);

        let z: BigComplex = "1.5 + 2.3I".parse().unwrap();
        afnear!(z.x.f64_decimal(), 1.5);
        afnear!(z.y.f64_decimal(), 2.3);
    }

    #[test]
    fn significant_digits() {
        use super::count_significant_digits;
        assert_eq!(count_significant_digits("1.5"), 2);
        assert_eq!(count_significant_digits("0.00123"), 3);
        assert_eq!(count_significant_digits("-0.0001000"), 4);
        assert_eq!(count_significant_digits("0"), 0);
        assert_eq!(count_significant_digits("-0"), 0);
    }

    #[test]
    fn precision_estimates() {
        use super::{count_significant_digits, estimate_precision};
        let real = "1.5";
        let imag = "2.3";
        assert_eq!(count_significant_digits(real), 2);
        assert_eq!(count_significant_digits(imag), 2);
        let prec = estimate_precision(real, imag);
        println!("Estimated precision for {real} + {imag}i: {prec} bits");
        assert_eq!(prec, 7);
    }
}
