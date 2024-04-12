// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use anyhow::ensure;

use super::userplotspec::{Location, Size};
use super::{PlotSpec, Point, Scalar};
use crate::{fractal, util::Rect};

use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

/// Machine-facing specification of a tile to plot
#[derive(Debug, Clone)]
pub struct TileSpec {
    /// Origin of this tile (bottom-left corner, smallest real/imaginary coefficients)
    origin: Point,
    /// Axes length for this tile
    axes: Point,
    /// Size in pixels of this tile
    size_in_pixels: Rect<u32>,
    /// If present, this tile is part of a larger plot; this is its Pixel offset within
    offset_within_plot: Option<Rect<u32>>,

    /// The selected algorithm
    algorithm: Arc<fractal::Instance>,
    /// Iteration limit
    max_iter: u32,
}

/// Method of splitting a tile
#[derive(Debug, Clone, Copy)]
pub enum SplitMethod {
    /// Full-width strips.
    RowsOfHeight(u32),
    // TODO Square
}

/// Canonicalised data about a plot.
/// For convenient construction, use ``From<&PlotSpec>``.
impl TileSpec {
    /// Constructor
    #[must_use]
    pub fn new(
        origin: Point,
        axes: Point,
        size_in_pixels: Rect<u32>,
        algorithm: &Arc<fractal::Instance>,
        max_iter: u32,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            offset_within_plot: None,
            algorithm: Arc::clone(algorithm),
            max_iter,
        }
    }
    /// Alternate constructor taking an offset
    #[must_use]
    pub fn new_with_offset(
        origin: Point,
        axes: Point,
        size_in_pixels: Rect<u32>,
        // If present, this tile is part of a larger plot; this is its Pixel offset (width, height) within
        offset_within_plot: Option<Rect<u32>>,
        algorithm: &Arc<fractal::Instance>,
        max_iter: u32,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            offset_within_plot,
            algorithm: Arc::clone(algorithm),
            max_iter,
        }
    }

    /// Splits this tile up into a number of smaller tiles, for parallelisation
    pub fn split(&self, how: SplitMethod, debug: u8) -> anyhow::Result<Vec<TileSpec>> {
        match how {
            SplitMethod::RowsOfHeight(row_height) => {
                let n_whole = self.height() / row_height;
                let maybe_last_height: Option<u32> = match self.height() % row_height {
                    0 => None,
                    other => Some(other),
                };

                // What is fixed about the subtiles?
                let strip_pixel_size = Rect::new(self.width(), row_height);
                let axes = Point {
                    re: self.axes.re,
                    im: self.axes.im * Scalar::from(row_height) / Scalar::from(self.height()),
                };
                // What varies as we go round the loop?
                let mut working_origin = self.origin;
                let origin_step = Point {
                    re: 0.0,
                    im: self.axes.im * Scalar::from(row_height) / Scalar::from(self.height()),
                };
                // Curveball: Pixel offsets are computed relative to top left, so we must invert the height dimension.
                // The first strip ends at the top, so starts one strip's height down from there.
                // We will start the height register at the top left point, which is where the first strip ENDS.
                let mut offset = Rect::<u32>::new(0, self.height());

                let mut output = Vec::<TileSpec>::with_capacity(n_whole as usize + 1);
                for i in 0..n_whole {
                    // Note we subtract the offset height before using it.
                    // This has the property that after the last whole strip, height is either 0, or is the height of the remainder strip.
                    offset.height -= row_height;
                    output.push(TileSpec::new_with_offset(
                        working_origin,
                        axes,
                        strip_pixel_size,
                        Some(offset),
                        &self.algorithm,
                        self.max_iter,
                    ));
                    if debug > 0 {
                        println!("tile {i} origin {working_origin} offset {offset}");
                    }
                    working_origin += origin_step;
                }
                if let Some(last_height) = maybe_last_height {
                    // There may be a slight imprecision when repeatedly adding small amounts.
                    // Therefore we recompute the last strip to take what's left of the overall axes.
                    let last_axes = Point {
                        re: self.axes.re,
                        im: self.axes.im + self.origin.im - working_origin.im,
                    };
                    ensure!(offset.height == last_height, "Unexpected remainder strip height ({}, expected {last_height}) - logic error?", offset.height);
                    offset.height = 0;
                    output.push(TileSpec::new_with_offset(
                        working_origin,
                        last_axes,
                        Rect::new(self.width(), last_height),
                        Some(offset),
                        &self.algorithm,
                        self.max_iter,
                    ));
                }
                // Finally: We have worked from the bottom to the top. Reverse the order for better aesthetics.
                output.reverse();
                Ok(output)
            }
        }
    }

    /// Automatically adjusts this spec to make the pixels square.
    /// This is done by growing the real or imaginary axis to suit.
    /// Obviously, you must call this before ``split()`` !
    /// Return: If we did anything, returns the new Axes value.
    pub fn auto_adjust_aspect_ratio(&mut self) -> anyhow::Result<Option<Point>> {
        let axes_aspect = self.axes.re / self.axes.im;
        let pixels_aspect = self.size_in_pixels.aspect_ratio();
        let ratio = pixels_aspect / axes_aspect;
        let centre = self.centre();
        if axes_aspect < pixels_aspect {
            // The requested pixel dimensions are too narrow.
            // Grow the plot in Real, maintaining its centre.
            ensure!(
                ratio > 1.0,
                "logic error; computed ratio {ratio} (expected >1)"
            );
            self.axes.re *= ratio;
            // Recompute origin to keep the same centre
            self.origin = centre - 0.5 * self.axes;
            Ok(Some(self.axes))
        } else if axes_aspect > pixels_aspect {
            // The requested pixel dimensions are too tall.
            // Grow the plot in Imaginary, maintaining its centre.
            ensure!(
                ratio < 1.0,
                "logic error; computed ratio {ratio} (expected <1)"
            );
            self.axes.im /= ratio;
            self.origin = centre - 0.5 * self.axes;
            Ok(Some(self.axes))
        } else {
            Ok(None) // nothing to do
        }
    }

    /// Computing accessor for the pixel size for this spec.
    #[must_use]
    pub fn pixel_size(&self) -> Point {
        Point {
            re: self.axes.re / Scalar::from(self.width()),
            im: self.axes.im / Scalar::from(self.height()),
        }
    }
    /// Computing accessor for the centre of this spec.
    #[must_use]
    pub fn centre(&self) -> Point {
        self.origin + 0.5 * self.axes
    }
    /// Accessor - Fractal origin i.e. smallest points in both axes; bottom-left point as drawn
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }
    /// Computing accessor - top left point as drawn (smallest real, largest imaginary)
    #[must_use]
    pub fn top_left(&self) -> Point {
        Point {
            re: self.origin.re,
            im: self.origin.im + self.axes.im,
        }
    }
    /// Computing accessor - bottom right point as drawn (largest real, smallest imaginary)
    #[must_use]
    pub fn bottom_right(&self) -> Point {
        Point {
            re: self.origin.re + self.axes.re,
            im: self.origin.im,
        }
    }

    /// Accessor
    #[must_use]
    pub fn axes(&self) -> Point {
        self.axes
    }
    /// Accessor - height in pixels
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size_in_pixels.height
    }
    /// Accessor - width in pixels
    #[must_use]
    pub fn width(&self) -> u32 {
        self.size_in_pixels.width
    }
    /// Accessor (clones reference)
    #[must_use]
    pub fn algorithm(&self) -> Arc<fractal::Instance> {
        Arc::clone(&self.algorithm)
    }
    /// Accessor
    #[must_use]
    pub fn offset_within_plot(&self) -> Option<Rect<u32>> {
        self.offset_within_plot
    }
    /// Accessor
    #[must_use]
    pub fn max_iter_requested(&self) -> u32 {
        self.max_iter
    }
}

const DEFAULT_AXIS_LENGTH: Scalar = 4.0;

impl From<&PlotSpec> for TileSpec {
    fn from(upd: &PlotSpec) -> Self {
        // Must compute axes first as origin may depend on them
        let axes: Point = match upd.axes {
            Size::AxesLength(l) => l,
            Size::PixelSize(p) => Point {
                re: p.re * Scalar::from(upd.width()),
                im: p.im * Scalar::from(upd.height()),
            },
            Size::ZoomFactor(zoom) => Point {
                re: DEFAULT_AXIS_LENGTH / zoom,
                im: (DEFAULT_AXIS_LENGTH / zoom) / upd.aspect_ratio(),
            },
        };
        let origin: Point = match upd.location {
            Location::Origin(o) => o,
            Location::Centre(c) => c - 0.5 * axes,
        };
        TileSpec {
            origin,
            axes,
            size_in_pixels: upd.size_in_pixels,
            offset_within_plot: None,
            algorithm: Arc::new(upd.algorithm),
            max_iter: upd.max_iter,
        }
    }
}

impl Display for TileSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}@{},axes={}", self.algorithm, self.origin, self.axes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        fractal::{
            self,
            tilespec::SplitMethod,
            userplotspec::{Location, Size},
            PlotSpec, Point, Scalar, TileSpec,
        },
        util::Rect,
    };
    use approx::assert_relative_eq;

    const ZERO: Point = Point { re: 0.0, im: 0.0 };
    const ONE: Point = Point { re: 1.0, im: 1.0 };
    const ONETWO: Point = Point { re: 1.0, im: 2.0 };
    const CENTI: Point = Point { re: 0.01, im: 0.01 };

    const MANDELBROT: fractal::Instance =
        fractal::Instance::Original(fractal::mandelbrot::Original {});

    const TD_ORIGIN_AXES: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::AxesLength(ONE),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 100,
        },
        algorithm: MANDELBROT,
        max_iter: 256,
    };
    const TD_ORIGIN_PIXELS: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::PixelSize(CENTI),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 100,
        },
        // this has the property that {width,height} * CENTI = { 1,1 }
        algorithm: MANDELBROT,
        max_iter: 256,
    };
    const TD_ORIGIN_ZOOM: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::ZoomFactor(1000.0),
        size_in_pixels: Rect::<u32> {
            width: 200,
            height: 100,
        },
        // note funky aspect ratio.
        // 4.0 default axis * zoom factor 1000 = 0.004 across
        // 200x100 pixels => (0.004,0.002) axes.
        algorithm: MANDELBROT,
        max_iter: 256,
    };
    const TD_CENTRE: PlotSpec = PlotSpec {
        location: Location::Centre(ONETWO),
        axes: Size::AxesLength(ONE),
        // centre(1,2) => origin (0.5,1.5)
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 100,
        },
        algorithm: MANDELBROT,
        max_iter: 256,
    };

    const TD_ORIGIN_ZOOM_AXES: Point = Point {
        re: 0.004,
        im: 0.002,
    };
    const TD_CENTRE_ORIGIN: Point = Point { re: 0.5, im: 1.5 };

    #[test]
    fn axes_pass_through() {
        let result = TileSpec::from(&TD_ORIGIN_AXES);
        assert_eq!(result.axes, ONE);
    }
    #[test]
    fn pixel_size_divides() {
        let result = TileSpec::from(&TD_ORIGIN_PIXELS);
        assert_eq!(result.axes, ONE);
    }
    #[test]
    fn aspect_axes() {
        assert_relative_eq!(TD_ORIGIN_ZOOM.aspect_ratio(), 2.0);
        let result = TileSpec::from(&TD_ORIGIN_ZOOM);
        assert_eq!(result.axes, TD_ORIGIN_ZOOM_AXES);
    }

    #[test]
    fn origin_pass_through() {
        let result = TileSpec::from(&TD_ORIGIN_AXES);
        assert_eq!(result.origin, ZERO);
    }
    #[test]
    fn centre_computed() {
        let result = TileSpec::from(&TD_CENTRE);
        assert_eq!(result.origin, TD_CENTRE_ORIGIN);
    }
    #[test]
    fn top_left_computed() {
        let ts = TileSpec::from(&TD_CENTRE);
        // centre(1,2) & axes (1,1) => top-left (0.5,2.5)
        let expected = Point { re: 0.5, im: 2.5 };
        assert_eq!(ts.top_left(), expected);
    }
    #[test]
    fn bottom_right_computed() {
        let ts = TileSpec::from(&TD_CENTRE);
        // centre(1,2) & axes (1,1) => top-left (1.5,1.5)
        let expected = Point { re: 1.5, im: 1.5 };
        assert_eq!(ts.bottom_right(), expected);
    }

    const TD_200H: PlotSpec = PlotSpec {
        location: Location::Centre(ZERO),
        axes: Size::AxesLength(ONE),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 200,
        },
        algorithm: MANDELBROT,
        max_iter: 256,
    };

    #[test]
    fn split_strips_no_remainder() {
        const TEST_HEIGHT: u32 = 10;
        let spec = TileSpec::from(&TD_200H);
        assert_eq!(
            spec.height() % TEST_HEIGHT,
            0,
            "This test requires a test spec that is a multiple of {TEST_HEIGHT} pixels high"
        );
        let result = spec
            .split(SplitMethod::RowsOfHeight(TEST_HEIGHT), 0)
            .unwrap();
        assert_eq!(
            result.len(),
            (spec.height() / TEST_HEIGHT) as usize,
            "Wrong number of output strips"
        );
        sanity_check_strips(&spec, &result, TEST_HEIGHT, None);
    }

    #[test]
    fn split_strips_with_remainder() {
        const TEST_HEIGHT: u32 = 11;
        let spec = TileSpec::from(&TD_200H);
        let remainder = spec.height() % TEST_HEIGHT;
        assert_ne!(
            remainder, 0,
            "This test requires a test spec that is not a multiple of {TEST_HEIGHT} pixels high"
        );
        let result = spec
            .split(SplitMethod::RowsOfHeight(TEST_HEIGHT), 0)
            .unwrap();
        assert_eq!(
            result.len(),
            1 + (spec.height() / TEST_HEIGHT) as usize,
            "Wrong number of output strips"
        );
        sanity_check_strips(&spec, &result, TEST_HEIGHT, Some(remainder));
    }

    fn sanity_check_strips(
        spec: &TileSpec,
        result: &Vec<TileSpec>,
        test_height: u32,
        remainder_height: Option<u32>,
    ) {
        // Sanity check the results
        let upper_corner = spec.origin() + spec.axes();

        let check_one = |ts: &TileSpec, remainder_height: Option<u32>| {
            let expected_axes_length = Point {
                re: spec.axes().re,
                im: spec.axes().im * Scalar::from(remainder_height.unwrap_or(test_height))
                    / Scalar::from(spec.height()),
            };

            // origin
            assert_relative_eq!(ts.origin().re, spec.origin().re);
            assert!(ts.origin().im >= spec.origin().im);
            assert!(
                ts.origin().im <= upper_corner.im,
                "subtile origin im is implausible; {} but upper corner is {}",
                ts.origin(),
                upper_corner
            );
            // axes
            assert_relative_eq!(ts.axes().re, expected_axes_length.re);
            assert_relative_eq!(ts.axes().im, expected_axes_length.im); // slippery in the remainder case!

            // pixel offset
            let offset = ts.offset_within_plot().unwrap();
            assert_eq!(offset.width, 0);
            assert!(offset.height <= spec.height());
            // pixel dimensions
            assert_eq!(ts.width(), spec.width());
            let expected_height = remainder_height.unwrap_or(test_height);
            assert_eq!(ts.height(), expected_height);

            // algorithm
            assert_eq!(ts.algorithm(), spec.algorithm());
        };

        if let Some(hh) = remainder_height {
            // First entry in the vector gets special handling as it's the remainder strip
            // (this is brittle!)
            check_one(result.first().unwrap(), Some(hh));
            for ts in &result[1..] {
                check_one(ts, None);
            }
        } else {
            for ts in result {
                check_one(ts, None);
            }
        }

        // check no overflow.
        // The last tile added - the FIRST in the output vector - is the topmost, so subject to the most accumulated error.
        let first: &TileSpec = result.first().unwrap();
        assert_relative_eq!(first.origin().im + first.axes().im, upper_corner.im);
    }

    #[test]
    fn aspect_1() {
        let mut ts = TileSpec::new(
            Point { re: -2.0, im: -2.0 },
            Point { re: 4.0, im: 4.0 },
            Rect::new(100, 100),
            &Arc::new(MANDELBROT),
            256,
        );
        assert!(ts.auto_adjust_aspect_ratio().is_ok_and(|v| v.is_none()));
    }

    #[test]
    fn aspect_gt_1() {
        let ts = TileSpec::new(
            Point { re: -2.0, im: -2.0 },
            Point { re: 4.0, im: 4.0 },
            Rect::new(200, 100),
            &Arc::new(MANDELBROT),
            256,
        );
        check_aspect(ts);
    }
    #[test]
    fn aspect_lt_1() {
        let ts = TileSpec::new(
            Point { re: -2.0, im: -2.0 },
            Point { re: 4.0, im: 4.0 },
            Rect::new(100, 200),
            &Arc::new(MANDELBROT),
            256,
        );
        check_aspect(ts);
    }
    fn check_aspect(mut ts: TileSpec) {
        let previous_centre = ts.centre();
        let res = ts.auto_adjust_aspect_ratio().unwrap();
        let new_axes = res.unwrap();
        // new_axes should be as reported
        assert_eq!(new_axes, ts.axes());
        // centre should be unchanged (0,0)
        assert_eq!(ts.centre(), previous_centre);
        // aspect ratio should now match pixel size
        let aspect = ts.axes().re / ts.axes().im;
        assert_relative_eq!(aspect, ts.size_in_pixels.aspect_ratio());
    }

    #[test]
    fn stringify() {
        let uut = TileSpec::new(
            Point::new(0.0, 0.5),
            Point::new(1.0, 2.0),
            Rect::new(200, 400),
            &Arc::new(fractal::framework::factory(
                fractal::framework::Selection::Original,
            )),
            256,
        );
        let result = uut.to_string();
        assert_eq!(result, "original@0+0.5i,axes=1+2i");
    }
}
