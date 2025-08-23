// Custom formatting code for floating-point types
// (c) 2024 Ross Younger

use num_complex::Complex;
use std::fmt;

/// A trait that works like display but formats numbers to a given number of significant figures
pub trait DisplaySignificantFigures {
    /// Formats a floating point number.
    /// @sf@ is the number of significant figures to output.
    /// (caution, this is a form of precision but not what Rust normally calls "precision").
    fn fmt_with_sf(&self, buf: &mut impl std::fmt::Write, sf: usize) -> fmt::Result;
}

impl DisplaySignificantFigures for f64 {
    fn fmt_with_sf(&self, buf: &mut impl std::fmt::Write, sf: usize) -> fmt::Result {
        write!(buf, "{}", to_string_with_precision(*self, sf, true))
    }
}

impl DisplaySignificantFigures for Complex<f64> {
    fn fmt_with_sf(&self, buf: &mut impl std::fmt::Write, sf: usize) -> fmt::Result {
        let sign_im = if self.im >= 0.0 { "+" } else { "" };
        write!(
            buf,
            "{}{sign_im}{}i",
            to_string_with_precision(self.re, sf, true),
            to_string_with_precision(self.im, sf, true)
        )
    }
}

/// A truncating version of Display
pub trait DisplayDecimalPlacesTrimmed {
    /// Formats a floating point number.
    /// @dp@ is the number of decimal places requested to output (what rust normally calls precision),
    /// but trailing zeroes will be trimmed.
    fn fmt_with_dp(&self, buf: &mut impl std::fmt::Write, dp: usize) -> fmt::Result;
}

impl DisplayDecimalPlacesTrimmed for f64 {
    fn fmt_with_dp(&self, buf: &mut impl std::fmt::Write, dp: usize) -> fmt::Result {
        write!(buf, "{}", to_string_with_dp(*self, dp, true))
    }
}

impl DisplayDecimalPlacesTrimmed for Complex<f64> {
    fn fmt_with_dp(&self, buf: &mut impl std::fmt::Write, dp: usize) -> fmt::Result {
        let sign_im = if self.im >= 0.0 { "+" } else { "" };
        write!(
            buf,
            "{}{sign_im}{}i",
            to_string_with_dp(self.re, dp, true),
            to_string_with_dp(self.im, dp, true)
        )
    }
}

/// Trims the trailing zeroes from a string
pub fn trim_trailing_zeroes(s: &mut String) {
    while s.len() > 1 && s.ends_with('0') {
        let _ = s.pop();
    }
    if s.len() > 1 && s.ends_with('.') {
        let _ = s.pop();
    }
}

/// Formats a floating point number.
/// The precision argument specifies the number of significant figures.
/// (This is not what Rust normally calls precision.)
fn to_string_with_precision(float: f64, precision: usize, strip_trailing_zeroes: bool) -> String {
    #![allow(clippy::cast_possible_truncation)]
    #![allow(clippy::cast_sign_loss)]
    // Based on https://stackoverflow.com/questions/60497397/how-do-you-format-a-float-to-the-first-significant-decimal-and-with-specified-pr

    // compute absolute value
    let a = float.abs();

    // if abs value is greater than 1, then precision becomes less than "standard"
    let precision = if a >= 1. {
        // reduce by number of digits, minimum 0
        let n = (1. + a.log10().floor()) as usize;
        precision.saturating_sub(n)
    // if precision is less than 1 (but non-zero), then precision becomes greater than "standard"
    } else if a > 0. {
        // increase number of digits
        let n = -(1. + a.log10().floor()) as usize;
        precision + n
    // special case for 0
    } else {
        0
    };

    // format with the given computed precision
    let mut result = format!("{float:.precision$}");
    if strip_trailing_zeroes {
        trim_trailing_zeroes(&mut result);
    }
    result
}

/// Formats a floating point number.
/// The precision argument specifies the number of decimal places.
fn to_string_with_dp(float: f64, precision: usize, strip_trailing_zeroes: bool) -> String {
    let mut result = format!("{float:.precision$}");
    if strip_trailing_zeroes {
        trim_trailing_zeroes(&mut result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayDecimalPlacesTrimmed, DisplaySignificantFigures, to_string_with_dp,
        to_string_with_precision,
    };
    use crate::fractal::maths::Point;

    #[test]
    fn sf_base() {
        assert_eq!(to_string_with_precision(1.5, 3, false), "1.50");
        assert_eq!(to_string_with_precision(1.5, 3, true), "1.5");
        assert_eq!(to_string_with_precision(1.2345, 3, false), "1.23");
        assert_eq!(to_string_with_precision(1.2399, 3, false), "1.24");
        assert_eq!(to_string_with_precision(1.49, 1, false), "1");
        assert_eq!(to_string_with_precision(1.49, 10, false), "1.490000000");
        assert_eq!(to_string_with_precision(1.49, 10, true), "1.49");
        assert_eq!(to_string_with_precision(1.049, 1, false), "1");
        assert_eq!(to_string_with_precision(-1.5, 3, false), "-1.50");
        assert_eq!(to_string_with_precision(-1.5, 3, true), "-1.5");
    }

    #[test]
    fn sf_float_trait() {
        let f = 1.2345;
        let mut buf = String::new();
        assert!(f.fmt_with_sf(&mut buf, 2).is_ok());
        assert_eq!(buf, "1.2");
    }

    #[test]
    fn sf_complex_trait() {
        let pt = Point::new(1.2345, 6.78901);
        let mut buf = String::new();
        assert!(pt.fmt_with_sf(&mut buf, 2).is_ok());
        assert_eq!(buf, "1.2+6.8i");

        let pt2 = Point::new(-1.2345, -6.78901);
        buf.clear();
        assert!(pt2.fmt_with_sf(&mut buf, 2).is_ok());
        assert_eq!(buf, "-1.2-6.8i");
    }

    #[test]
    fn dp_base() {
        assert_eq!(to_string_with_dp(1.5, 3, false), "1.500");
        assert_eq!(to_string_with_dp(1.5, 3, true), "1.5");
        assert_eq!(to_string_with_dp(1.5, 1, false), "1.5");
        assert_eq!(to_string_with_dp(1.5, 1, true), "1.5");

        assert_eq!(to_string_with_dp(1., 1, true), "1");
        assert_eq!(to_string_with_dp(1., 1, false), "1.0");

        assert_eq!(to_string_with_dp(1.2345, 1, false), "1.2");
        assert_eq!(to_string_with_dp(1.2345, 3, false), "1.234");
        assert_eq!(to_string_with_dp(1.2345, 4, false), "1.2345");

        assert_eq!(to_string_with_dp(-1.5, 3, false), "-1.500");
        assert_eq!(to_string_with_dp(-1.5, 3, true), "-1.5");
    }

    #[test]
    fn dp_float_trait() {
        let f = 1.2345;
        let mut buf = String::new();
        assert!(f.fmt_with_dp(&mut buf, 10).is_ok());
        assert_eq!(buf, "1.2345");
    }
    #[test]
    fn dp_complex_trait() {
        let pt = Point::new(1.2345, 6.78901);
        let mut buf = String::new();
        assert!(pt.fmt_with_dp(&mut buf, 2).is_ok());
        assert_eq!(buf, "1.23+6.79i");

        let pt2 = Point::new(-1.2345, -6.78901);
        buf.clear();
        assert!(pt2.fmt_with_dp(&mut buf, 2).is_ok());
        assert_eq!(buf, "-1.23-6.79i");
    }
}
