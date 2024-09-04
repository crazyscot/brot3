// Info string ("heads up display") & formatting
// (c) 2024 Ross Younger

use std::{cell::RefCell, cmp};

use brot3_engine::fractal::{Algorithm as _, Point, Scalar};

use crate::{
    components::{InfoDisplayData, MainUI},
    types::PixelCoordinate,
    World,
};

// HELPER FUNCTIONS =================================================================

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

// SLINT INTERACTION ================================================================

pub(crate) fn update_info_display(world_: &RefCell<World>, ui: &MainUI) {
    let world = world_.borrow();
    let window_dimensions = world.visible_dimensions();
    let world_size_pixels = world.world_size();
    let algorithm_instance =
        brot3_engine::fractal::factory(brot3_engine::fractal::Selection::Original); // TODO use algorithm from spec
    let fractal_size = algorithm_instance.default_axes();

    #[allow(clippy::cast_precision_loss)]
    // window_dimensions is small (screen resolution) so no precision loss.
    // world_size_pixels is a power of 2 so no precision loss.
    let visible_axes_length = brot3_engine::fractal::Point::new(
        window_dimensions.x as Scalar * fractal_size.re / world_size_pixels as Scalar,
        window_dimensions.y as Scalar * fractal_size.im / world_size_pixels as Scalar,
    );

    #[allow(clippy::cast_precision_loss)]
    // world_size_pixels is a power of 2 so no precision loss.
    let complex_pixel_size: Point = fractal_size.unscale(world_size_pixels as Scalar);

    let top_left_pixel = world.visible_origin();
    let bottom_left_pixel = PixelCoordinate {
        x: top_left_pixel.x,
        y: top_left_pixel.y + world.visible_height - 1,
    };
    // Location of the bottom left pixel, expressed as a vector relative to the bottom left of the world
    let bottom_left_offset: PixelCoordinate = PixelCoordinate {
        x: top_left_pixel.x,
        y: world.world_size() - bottom_left_pixel.y - 1,
    };
    // Offset of the bottom left pixel, in complex units, from the fractal origin
    #[allow(clippy::cast_precision_loss)]
    // Maximum pixel size, and hence bottom_left_offset, are limited to fit within f64 mantissa (TECHDEBT)
    let origin_offset = Point::new(
        complex_pixel_size.re * bottom_left_offset.x as Scalar,
        complex_pixel_size.im * bottom_left_offset.y as Scalar,
    );
    let origin_absolute = origin_offset - algorithm_instance.default_axes().unscale(2.)
        + algorithm_instance.default_centre();
    let centre_absolute = origin_absolute + visible_axes_length.unscale(2.);

    let axes = {
        let axes_precision = axes_precision_for_canvas(window_dimensions);
        let real_axis = format_float_with_precision("", visible_axes_length.re, axes_precision);
        let imag_axis = format_float_with_precision("+", visible_axes_length.im, axes_precision);
        format!("{real_axis}{imag_axis}i")
    };

    let position_dp = decimal_places_for_axes(window_dimensions, visible_axes_length);
    let origin = {
        let origin_real = format_float_fixed("", origin_absolute.re, position_dp);
        let origin_imag = format_float_fixed("+", origin_absolute.im, position_dp);
        format!("{origin_real}{origin_imag}i")
    };

    let centre = {
        let real = format_float_fixed("", centre_absolute.re, position_dp);
        let imag = format_float_fixed("+", centre_absolute.im, position_dp);
        format!("{real}{imag}i")
    };

    let z: u64 = 1 << (world.zoom_level - 1);
    let (z_mantissa, z_exponent) = if z < 1000 {
        (format!("{z}"), 0)
    } else {
        let part = format!("{z:.3e}");
        let mut bits = part.split('e');
        let mantissa = bits.next().unwrap_or("").to_string();
        let exp = bits.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        (mantissa, exp)
    };

    let info = InfoDisplayData {
        algorithm: "Original".into(),  // TODO this comes from alg spec
        colourer: "LogRainbow".into(), // TODO from alg spec
        max_iter: crate::types::UI_TEMP_MAXITER.into(), // TODO from alg spec
        origin: origin.into(),
        centre: centre.into(),
        axes: axes.into(),
        zoom_mantissa: z_mantissa.into(),
        zoom_exponent10: z_exponent,
    };
    ui.set_info_data(info);
}

// TEST =============================================================================

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
