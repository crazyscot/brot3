// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use super::userplotspec::{Location, Size};
use super::{FractalInstance, PlotSpec, Point, Scalar};
use crate::util::Rect;

use std::fmt::{self, Display, Formatter};

/// Machine-facing specification of a tile to plot
#[derive(Debug, Clone, Copy)]
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
    algorithm: FractalInstance,
}

/// Method of splitting a tile
#[derive(Debug, Clone, Copy)]
pub enum SplitMethod {
    /// Full-width strips
    Rows(u32),
    // TODO Square
}

/// Canonicalised data about a plot.
/// For convenient construction, use From<&``UserPlotData``>.
impl TileSpec {
    /// Computes the pixel size for this spec.
    #[must_use]
    pub fn pixel_size(&self) -> Point {
        Point {
            re: self.axes.re / Scalar::from(self.width()),
            im: self.axes.im / Scalar::from(self.height()),
        }
    }

    /// Constructor
    #[must_use]
    pub fn new(
        origin: Point,
        axes: Point,
        size_in_pixels: Rect<u32>,
        algorithm: FractalInstance,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            offset_within_plot: None,
            algorithm,
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
        algorithm: FractalInstance,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            offset_within_plot,
            algorithm,
        }
    }

    /// Splits this tile up into a number of smaller tiles, for parallelisation
    #[must_use]
    pub fn split(&self, how: SplitMethod) -> Vec<TileSpec> {
        match how {
            SplitMethod::Rows(row_height) => {
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
                let mut offset = Rect::<u32>::default();

                let mut output = Vec::<TileSpec>::with_capacity(n_whole as usize);
                for _ in 0..n_whole {
                    output.push(TileSpec::new_with_offset(
                        working_origin,
                        axes,
                        strip_pixel_size,
                        Some(offset),
                        self.algorithm,
                    ));
                    working_origin += origin_step;
                    offset.height += row_height;
                }
                if let Some(last_height) = maybe_last_height {
                    // There may be a slight imprecision when repeatedly adding small amounts.
                    // Therefore we recompute the last strip to take what's left of the overall axes.
                    let last_axes = Point {
                        re: self.axes.re,
                        im: self.axes.im + self.origin.im - working_origin.im,
                    };
                    output.push(TileSpec::new_with_offset(
                        working_origin,
                        last_axes,
                        Rect::new(self.width(), last_height),
                        Some(offset),
                        self.algorithm,
                    ));
                }
                // Finally: We have worked from the bottom to the top. Reverse the order for better aesthetics.
                output.reverse();
                output
            }
        }
    }

    /// Accessor
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }
    /// Accessor
    #[must_use]
    pub fn axes(&self) -> Point {
        self.axes
    }
    /// Accessor
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size_in_pixels.height
    }
    /// Accessor
    #[must_use]
    pub fn width(&self) -> u32 {
        self.size_in_pixels.width
    }
    /// Accessor
    #[must_use]
    pub fn algorithm(&self) -> FractalInstance {
        self.algorithm
    }
    /// Accessor
    #[must_use]
    pub fn offset_within_plot(&self) -> Option<Rect<u32>> {
        self.offset_within_plot
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
            algorithm: upd.algorithm,
        }
    }
}

impl Display for TileSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "origin={} axes={}", self.origin, self.axes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        fractal::{
            self,
            tilespec::SplitMethod,
            userplotspec::{Location, Size},
            FractalInstance, PlotSpec, Point, Scalar, TileSpec,
        },
        util::Rect,
    };
    use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};

    const ZERO: Point = Point { re: 0.0, im: 0.0 };
    const ONE: Point = Point { re: 1.0, im: 1.0 };
    const ONETWO: Point = Point { re: 1.0, im: 2.0 };
    const CENTI: Point = Point { re: 0.01, im: 0.01 };

    const MANDELBROT: FractalInstance = FractalInstance::Original(fractal::mandelbrot::Original {});

    const TD_ORIGIN_AXES: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::AxesLength(ONE),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 100,
        },
        algorithm: MANDELBROT,
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
        assert_f64_near!(TD_ORIGIN_ZOOM.aspect_ratio(), 2.0);
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

    const TD_200H: PlotSpec = PlotSpec {
        location: Location::Centre(ZERO),
        axes: Size::AxesLength(ONE),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 200,
        },
        algorithm: MANDELBROT,
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
        let result = spec.split(SplitMethod::Rows(TEST_HEIGHT));
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
        let result = spec.split(SplitMethod::Rows(TEST_HEIGHT));
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
            assert_f64_near!(ts.origin().re, spec.origin().re);
            assert!(ts.origin().im >= spec.origin().im);
            assert!(
                ts.origin().im <= upper_corner.im,
                "subtile origin im is implausible; {} but upper corner is {}",
                ts.origin(),
                upper_corner
            );
            // axes
            assert_f64_near!(ts.axes().re, expected_axes_length.re);
            assert_f64_near!(ts.axes().im, expected_axes_length.im, 150); // slippery in the remainder case!

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
        assert_f64_near!(first.origin().im + first.axes().im, upper_corner.im);
    }
}
