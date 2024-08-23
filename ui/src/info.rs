// Info string ("heads up display") formatting
// (c) 2024 Ross Younger

use std::cmp;

use brot3_engine::fractal::{Point, Scalar};

use crate::types::PixelCoordinate;

/// Computes the decimal precision (number of significant figures) required for a given canvas size.
/// Rationale: If a change in axes would move us <1 pixel it has no visible effect.
pub(crate) fn axes_precision_for_canvas(canvas_size: PixelCoordinate) -> usize {
    #![allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    (cmp::max(canvas_size.x, canvas_size.y) as f64)
        .log10()
        .ceil() as usize
}

/// Computes the number of decimal places required for a given canvas and axes size.
/// Rationale: If a change in position would move us <1 pixel it has no visible effect.
pub(crate) fn decimal_places_for_axes(canvas_size: PixelCoordinate, axes_length: Point) -> usize {
    #![allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let pixel_size = Point::new(
        axes_length.re / canvas_size.x as Scalar,
        axes_length.im / canvas_size.y as Scalar,
    );
    (-f64::max(pixel_size.re, pixel_size.im).log10()).ceil() as usize
}

/// Formats a float with a given number of significant figures (decimal precision).
/// `positive` is prepended to numbers >= 0.0.
/// `precision` is the required number of significant figures.
pub(crate) fn format_float_with_precision(
    positive_prefix: &str,
    n: f64,
    precision: usize,
) -> String {
    let strnum = if n != 0. && n.abs() < 0.001 {
        format!("{n:.precision$e}")
    } else {
        format!("{n:.precision$}")
    };
    if n >= 0. {
        format!("{positive_prefix}{strnum}")
    } else {
        strnum
    }
}

/// Formats a float with a given (fixed) number of decimal places.
/// `positive_prefix` is prepended to numbers >= 0.0.
pub(crate) fn format_float_fixed(positive_prefix: &str, n: f64, decimal_places: usize) -> String {
    let strnum = format!("{n:.decimal_places$}");
    if n >= 0. {
        format!("{positive_prefix}{strnum}")
    } else {
        strnum
    }
}

#[cfg(test)]
mod tests {
    use crate::types::PixelCoordinate;
    use brot3_engine::fractal::Point;

    #[test]
    fn axes_precision() {
        let canvas = PixelCoordinate { x: 1920, y: 1080 };
        assert_eq!(super::axes_precision_for_canvas(canvas), 4);
    }
    #[test]
    fn decimal_places() {
        let canvas = PixelCoordinate { x: 1920, y: 1080 };
        let axes = Point::new(0.1, 0.1);
        assert_eq!(super::decimal_places_for_axes(canvas, axes), 5);
    }
    #[test]
    fn float_formats() {
        let uut = super::format_float_with_precision;
        // Positive, negative and zero
        assert_eq!(uut("+", 0.123, 4), "+0.1230");
        assert_eq!(uut("", 0.123, 4), "0.1230");
        assert_eq!(uut("+", -0.123, 4), "-0.1230");
        assert_eq!(uut("", -0.123, 4), "-0.1230");
        assert_eq!(uut("+", 0., 4), "+0.0000");

        // Decimal places
        assert_eq!(uut("", 1.234, 0), "1");
        assert_eq!(uut("", 1.234, 1), "1.2");
        assert_eq!(uut("", 1.5, 0), "2");

        // Scientific notation
        assert_eq!(uut("+", 0.000_123_4, 3), "+1.234e-4");
        assert_eq!(uut("+", 0.000_123_4, 1), "+1.2e-4");
        assert_eq!(uut("", 0.000_123_4, 0), "1e-4");
    }
    #[test]
    fn float_fixed() {
        let uut = super::format_float_fixed;
        assert_eq!(uut("+", 12., 5), "+12.00000");
        assert_eq!(uut("+", 12., 0), "+12");
        assert_eq!(uut("", 1.2345, 2), "1.23");
        assert_eq!(uut("", 1.987, 2), "1.99");
    }
}
