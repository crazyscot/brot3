// Info string ("heads up display") & formatting
// (c) 2024 Ross Younger

use std::{cell::RefCell, cmp, collections::BTreeMap, rc::Rc, str::FromStr as _};

use brot3_engine::{
    colouring,
    fractal::{self, Algorithm as _, Point, Scalar},
    util::listable,
};
use slint::{SharedString, VecModel};

use crate::{
    components::{InfoDisplayData, MainUI},
    types::PixelCoordinate,
    State, World,
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

#[derive(Default)]
pub(crate) struct StringCache {
    fractals: BTreeMap<fractal::Selection, SharedString>,
    colourers: BTreeMap<colouring::Selection, SharedString>,
}

pub(crate) fn update_info_display(
    world_: &RefCell<World>,
    ui: &MainUI,
    cache: &RefCell<StringCache>,
) {
    let world = world_.borrow();
    let window_dimensions = world.visible_dimensions();
    let world_size_pixels = world.world_size();
    let algorithm_instance = world.active_algspec.algorithm;
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

    let mut cache = cache.borrow_mut();
    let algorithm = cache
        .fractals
        .entry(world.active_algspec.algorithm.into())
        .or_insert_with(|| {
            let str: &'static str = world.active_algspec.algorithm.into();
            SharedString::from(str)
        })
        .clone();
    let colourer = cache
        .colourers
        .entry(world.active_algspec.colourer.into())
        .or_insert_with(|| {
            let str: &'static str = world.active_algspec.colourer.into();
            SharedString::from(str)
        })
        .clone();

    #[allow(clippy::cast_possible_wrap)]
    let info = InfoDisplayData {
        algorithm,
        colourer,
        max_iter: world.active_algspec.max_iter as _,
        origin: origin.into(),
        centre: centre.into(),
        axes: axes.into(),
        zoom_mantissa: z_mantissa.into(),
        zoom_exponent10: z_exponent,
    };
    ui.set_info_data(info);
}

pub(crate) fn populate_dropdowns(state: &Rc<State>) {
    let default = crate::types::default_algorithm();

    let avail_fractals = listable::list_original_case::<brot3_engine::fractal::Selection>()
        .map(|s| SharedString::from(s.name))
        .collect::<Vec<_>>();
    state
        .main_ui
        .set_fractals_available(slint::ModelRc::new(VecModel::from(avail_fractals)));
    state.main_ui.set_fractal_selection(
        brot3_engine::fractal::Selection::from(default.algorithm)
            .to_string()
            .into(),
    );

    let avail_colourers = listable::list_original_case::<brot3_engine::colouring::Selection>()
        .map(|s| SharedString::from(s.name))
        .collect::<Vec<_>>();
    state
        .main_ui
        .set_colourers_available(slint::ModelRc::new(VecModel::from(avail_colourers)));
    state.main_ui.set_colourer_selection(
        brot3_engine::colouring::Selection::from(default.colourer)
            .to_string()
            .into(),
    );

    let state_weak = Rc::downgrade(state);
    state.main_ui.on_fractal_selected(move |selection| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        let mut new_algspec = world.active_algspec;
        new_algspec.algorithm = brot3_engine::fractal::Instance::from_str(selection.as_str())
            .unwrap_or_else(|e| {
                eprintln!("{e}: {selection}");
                world.active_algspec.algorithm
            });
        world.reset_view_2(Some(new_algspec));
        drop(world);
        state.do_poll();
    });

    let state_weak = Rc::downgrade(state);
    state.main_ui.on_colourer_selected(move |selection| {
        let state = state_weak.upgrade().unwrap();
        let mut world = state.world.borrow_mut();
        let mut new_algspec = world.active_algspec;
        new_algspec.colourer = brot3_engine::colouring::Instance::from_str(selection.as_str())
            .unwrap_or_else(|e| {
                eprintln!("{e}: {selection}");
                world.active_algspec.colourer
            });
        world.reset_view_2(Some(new_algspec));
        drop(world);
        state.do_poll();
    });
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
