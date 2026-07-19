//! Arbitrary precision complex numbers, powered by `dashu::float::FBig`

use std::str::FromStr;

use dashu_float::{DBig, FBig};
use easy_cast::{Cast as _, Conv as _, ConvFloat as _};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

use crate::CBig;

/// Extension trait for [`CBig`] to provide convenience functions.
///
/// Internally, brot3 uses the `custom_serde` module to serialize, for example:
/// ```skip
///         #[serde(with = "crate::bignum::custom_serde")]
///         pub some_field: CBig,
/// ```
pub trait BigComplexExt {
    type ParseErr;
    fn try_new<T>(re: T, im: T) -> Result<Self, <FBig as TryFrom<T>>::Error>
    where
        Self: Sized,
        FBig: std::convert::TryFrom<T>;
    fn as_vec2(&self) -> crate::Vec2;
    fn as_dvec2(&self) -> crate::DVec2;
    fn try_from_dvec2(value: crate::DVec2) -> Result<Self, <FBig as TryFrom<f64>>::Error>
    where
        Self: Sized;
    fn add_assign_dvec2(
        &mut self,
        other: crate::DVec2,
    ) -> Result<(), <FBig as TryFrom<f64>>::Error>
    where
        Self: Sized;
    /// Custom parser for complex numbers in the format used by brot3
    fn from_str_b3(s: &str) -> Result<Self, Self::ParseErr>
    where
        Self: Sized;
    /// Custom conversion to string for complex numbers in the format used by brot3
    fn to_str_b3(&self) -> String;
}

impl BigComplexExt for crate::CBig {
    type ParseErr = ParseError;

    fn as_vec2(&self) -> crate::Vec2 {
        let (x, y) = (self.re(), self.im());
        crate::vec2(x.to_f32().value(), y.to_f32().value())
    }

    fn as_dvec2(&self) -> crate::DVec2 {
        let (x, y) = (self.re(), self.im());
        crate::dvec2(x.to_f64().value(), y.to_f64().value())
    }

    fn try_from_dvec2(value: crate::DVec2) -> Result<Self, <FBig as TryFrom<f64>>::Error> {
        Self::try_new(value.x, value.y)
    }

    fn try_new<T>(re: T, im: T) -> Result<Self, <FBig as TryFrom<T>>::Error>
    where
        Self: Sized,
        FBig: std::convert::TryFrom<T>,
    {
        let re = FBig::try_from(re)?;
        let im = FBig::try_from(im)?;
        Ok(Self::from_parts(re, im))
    }

    fn add_assign_dvec2(&mut self, other: crate::DVec2) -> Result<(), <FBig as TryFrom<f64>>::Error>
    where
        Self: Sized,
    {
        let temp = Self::try_from_dvec2(other)?;
        *self += temp;
        Ok(())
    }

    fn from_str_b3(s: &str) -> Result<Self, Self::ParseErr> {
        let (real_str, imag_str, precision) = parse_bigcomplex_input(s)?;
        let re = if real_str.is_empty() {
            FBig::ZERO
        } else {
            fbig_from_str(&real_str).with_precision(precision).value()
        };
        let im = if imag_str.is_empty() {
            FBig::ZERO
        } else {
            fbig_from_str(&imag_str).with_precision(precision).value()
        };
        Ok(CBig::from_parts(re, im))
    }

    fn to_str_b3(&self) -> String {
        use dashu::base::Sign;
        let (re, im) = self.clone().into_parts();
        let re = re.to_decimal().value();
        let sign = im.sign();
        let im = im.to_decimal().value();
        match sign {
            Sign::Negative => format!("{re} - {}i", -im),
            Sign::Positive => format!("{re} + {im}i"),
        }
    }
}

/// Roughly creates a [`CBig`] from a pair of inputs.
/// Intended for testing.
///
/// # Panics
/// If the numeric conversion failed
#[macro_export]
macro_rules! make_bigcomplex {
    ($x: expr, $y: expr) => {{
        use $crate::BigComplexExt;
        $crate::CBig::try_new($x, $y).unwrap()
    }};
}

/// Error type for parsing [`CBig`] from strings.
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

pub(crate) mod custom_serde {
    use serde::{Deserialize, Serialize};

    use super::{CBig, FBig};

    #[derive(Clone, Serialize, Deserialize, PartialEq)]
    struct Wrapper {
        #[serde(
            serialize_with = "super::serialize_fbig",
            deserialize_with = "super::deserialize_fbig",
            alias = "x"
        )]
        pub re: FBig,
        #[serde(
            serialize_with = "super::serialize_fbig",
            deserialize_with = "super::deserialize_fbig",
            alias = "y"
        )]
        pub im: FBig,
    }

    pub(crate) fn serialize<S: serde::Serializer>(
        value: &CBig,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let (re, im) = value.clone().into_parts();
        Wrapper { re, im }.serialize(serializer)
    }
    pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<CBig, D::Error> {
        let wrap = Wrapper::deserialize(deserializer)?;
        Ok(CBig::from_parts(wrap.re, wrap.im))
    }
}

fn serialize_fbig<S: Serializer>(value: &FBig, serializer: S) -> Result<S::Ok, S::Error> {
    (
        value.repr().significand().to_string(),
        value.repr().exponent(),
    )
        .serialize(serializer)
}
fn deserialize_fbig<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<FBig, D::Error> {
    let (sig_str, exp): (String, isize) = Deserialize::deserialize(deserializer)?;
    let sig = dashu::integer::IBig::from_str(&sig_str).map_err(serde::de::Error::custom)?;
    Ok(FBig::from_parts(sig, exp))
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

/// Parse a decimal string into an arbitrary-precision [`FBig`].
/// Preserves all significant digits in the string.
///
/// # Panics
/// If the string is not a valid decimal number.
#[must_use]
fn fbig_from_str(s: &str) -> FBig {
    DBig::from_str(s)
        .expect("failed to parse decimal string")
        .with_base::<2>()
        .value()
        .with_rounding::<dashu_float::round::mode::Zero>()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use dashu_float::round::mode::Zero;
    use derive_more::Display;
    use float_eq::assert_float_eq;
    use serde::{Deserialize, Serialize};

    use super::{BigComplexExt as _, CBig, FBig};

    #[test]
    fn exercise() {
        let c1 = make_bigcomplex!(1.0, 2.0);
        let c2 = CBig::try_new(3.0, 4.0).unwrap();
        let z = c1 + &c2;
        let z = z + c2;
        assert_eq!(z, make_bigcomplex!(7.0, 10.0));
        let a = make_bigcomplex!(123, 456);
        let b = a.clone() - a; // these are bignums, they do not support Copy
        assert_eq!(b, CBig::<Zero>::ZERO);
        assert_eq!(z.clone() - &z, CBig::<Zero>::ZERO);
    }

    /// Creates a [`crate::CBig`] from a pair of decimal strings with full precision
    macro_rules! make_bigcomplex_str {
        ($x: expr, $y: expr) => {{
            let x = super::fbig_from_str($x);
            let y = super::fbig_from_str($y);
            $crate::CBig::from_parts(x, y)
        }};
    }

    #[derive(Serialize, Deserialize, Debug, Display, PartialEq)]
    struct TestWrapper {
        #[serde(with = "super::custom_serde")]
        pub z: CBig,
    }

    #[test]
    fn serde_roundtrip() {
        let z = TestWrapper {
            z: make_bigcomplex_str!("-1.378186747593672212", "-0.0177134138869923"),
        };
        let json = serde_json::to_string(&z).expect("serialization failed");
        println!("z: {z}");
        println!("JSON: {json}");
        let z2: TestWrapper = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(z, z2);
    }
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

    struct StringWrap(CBig);
    impl std::str::FromStr for StringWrap {
        type Err = super::ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            CBig::from_str_b3(s).map(Self)
        }
    }
    impl std::fmt::Display for StringWrap {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.to_str_b3())
        }
    }

    #[test]
    fn from_str_full_complex() {
        // Standard format with spaces
        let z: StringWrap = "1.5 + 2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);

        // Standard format without spaces
        let z: StringWrap = "1.5+2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);

        // Negative imaginary
        let z: StringWrap = "1.5 - 2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), -2.3);

        // Both negative
        let z: StringWrap = "-1.5-2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), -1.5);
        afnear!(y.f64_decimal(), -2.3);

        // Leading positive sign
        let z: StringWrap = "+1.5+2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);
    }

    #[test]
    fn from_str_bare_real() {
        // Positive real
        let z: StringWrap = "1.5".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 0.0);

        // Negative real
        let z: StringWrap = "-2.3".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), -2.3);
        afnear!(y.f64_decimal(), 0.0);

        // Integer real
        let z: StringWrap = "42".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 42.0);
        afnear!(y.f64_decimal(), 0.0);
    }

    #[test]
    fn from_str_bare_imaginary() {
        // Simple imaginary
        let z: StringWrap = "2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 2.3);

        // Negative imaginary
        let z: StringWrap = "-2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), -2.3);

        // Unit imaginary (implicit 1)
        let z: StringWrap = "i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 1.0);

        // Negative unit imaginary
        let z: StringWrap = "-i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), -1.0);
    }

    #[test]
    fn from_str_edge_cases() {
        // Zero
        let z: StringWrap = "0".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 0.0);

        let z: StringWrap = "0i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 0.0);

        // Zero + zero i
        let z: StringWrap = "0+0i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 0.0);

        // With extra spaces
        let z: StringWrap = "1.5  +  2.3i".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);

        // Leading/trailing spaces
        let z: StringWrap = "  1.5 + 2.3i  ".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);
    }

    #[test]
    fn from_str_roundtrip_display() {
        let original_strs = vec!["1.5+2.3i", "1.5-2.3i", "-1.5-2.3i"];

        for original_str in original_strs {
            let z: StringWrap = original_str.parse().unwrap();
            let displayed = z.to_string();
            let z2: StringWrap = displayed.parse().unwrap();
            // Check that values match (with reasonable precision from binary representation)
            let (x1, y1) = z.0.into_parts();
            let (x2, y2) = z2.0.into_parts();
            afnear!(x1.f64_decimal(), x2.f64_decimal());
            afnear!(y1.f64_decimal(), y2.f64_decimal());
        }
    }

    #[test]
    fn from_str_precision_preservation() {
        // Long decimal - should preserve significant digits
        let z: StringWrap = "1.123456789012345678901234567890+2.987654321098765432109876543210i"
            .parse()
            .unwrap();

        // Check that precision was set (not exact equality due to rounding, but close)
        let prec = z.0.precision();
        assert!(
            prec > 50,
            "Real precision should be > 50 bits for 30 decimal digits"
        );
        assert!(
            prec > 50,
            "Imaginary precision should be > 50 bits for 30 decimal digits"
        );
    }

    #[test]
    fn from_str_errors() {
        // These should all return parse errors, not panics
        use super::ParseError;

        // Empty string
        let result: Result<StringWrap, _> = "".parse();
        assert!(matches!(result, Err(ParseError::MissingComplex)));

        // Only operators/signs
        let result: Result<StringWrap, _> = "+-".parse();
        assert!(result.is_err());

        // Invalid real part with double dot
        let result: Result<StringWrap, _> = "1.5.5+2.3i".parse();
        assert!(result.is_err());

        // Invalid imaginary part with double dot
        let result: Result<StringWrap, _> = "1.5+2.5.3i".parse();
        assert!(result.is_err());
    }

    #[test]
    fn from_str_uppercase_i() {
        // Should work with uppercase I as well
        let z: StringWrap = "2.3I".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 0.0);
        afnear!(y.f64_decimal(), 2.3);

        let z: StringWrap = "1.5 + 2.3I".parse().unwrap();
        let (x, y) = z.0.into_parts();
        afnear!(x.f64_decimal(), 1.5);
        afnear!(y.f64_decimal(), 2.3);
    }
}
