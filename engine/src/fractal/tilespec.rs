// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use anyhow::ensure;
use num_complex::Complex;

use super::{Location, Point, Scalar, Size};
use crate::{colouring, fractal, util::Rect};

use std::{
    fmt::{self, Display, Formatter},
    hash::Hash,
    sync::Arc,
};

const DEFAULT_AXIS_LENGTH: Scalar = 4.0;

/// Specification of the algorithmic part of a tile to plot
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlgorithmSpec {
    /// The selected algorithm
    pub algorithm: fractal::Instance,
    /// Iteration limit
    pub max_iter: u32,
    /// The selected colourer
    pub colourer: colouring::Colourer,
}

impl AlgorithmSpec {
    /// Standard constructor
    #[must_use]
    pub fn new(
        algorithm: fractal::Instance,
        max_iter: u32,
        colourer: colouring::Colourer,
    ) -> AlgorithmSpec {
        AlgorithmSpec {
            algorithm,
            max_iter,
            colourer,
        }
    }
    /// Syntactic sugar constructor (wraps new struct in an Arc)
    #[must_use]
    pub fn new_arc(
        algorithm: fractal::Instance,
        max_iter: u32,
        colourer: colouring::Colourer,
    ) -> Arc<AlgorithmSpec> {
        Arc::new(Self::new(algorithm, max_iter, colourer))
    }

    /// Can we use plotted tile data from this spec to seed some other spec?
    #[must_use]
    pub fn can_recompute(&self, other: &AlgorithmSpec) -> bool {
        self.algorithm == other.algorithm
    }
}

// this is a special-case implementation of PartialEq for cacheing support
fn alg_specs_are_equivalent(a1: &Arc<AlgorithmSpec>, a2: &Arc<AlgorithmSpec>) -> bool {
    a1.algorithm == a2.algorithm && a1.max_iter == a2.max_iter
}

/// Machine-facing specification of a tile to plot.
/// <br/><b>Note</b> Equality of this struct is not strictly well defined. Floating point types normally only implement ``PartialEq`` (since NaN != NaN),
/// and even ignoring that the inaccuracies within floats can lead to unexpected results.
/// However:
/// 1. We don't make use of NaN so can neglect that case.
/// 2. We are genuinely only comparing tilespecs for identicality, not recreating them which might lead to inaccuracies, so can neglect that case.
///
/// Therefore it is safe to naively treat f64s as bags of bits in implementing Eq and Hash.
#[derive(Debug, Clone)]
pub struct TileSpec {
    /// Origin of this tile (bottom-left corner, smallest real/imaginary coefficients)
    origin: Point,
    /// Axes length for this tile
    axes: Point,
    /// Size in pixels of this tile
    size_in_pixels: Rect<u32>,
    /// The selected algorithm, colourer and parameters
    alg_spec: Arc<AlgorithmSpec>,
    /// If present, this tile is a strip of a larger plot; this is its y offset within
    y_offset: Option<u32>,
}

/// Canonicalised specification of a plot
impl TileSpec {
    /// Constructor
    #[must_use]
    pub fn new(
        location: Location,
        size: Size,
        size_in_pixels: Rect<u32>,
        algorithm: fractal::Instance,
        max_iter: u32,
        colourer: colouring::Colourer,
    ) -> TileSpec {
        // Must compute axes first as origin may depend on them
        let axes: Point = match size {
            Size::AxesLength(l) => l,
            Size::PixelSize(p) => Point {
                re: p.re * Scalar::from(size_in_pixels.width),
                im: p.im * Scalar::from(size_in_pixels.height),
            },
            Size::ZoomFactor(zoom) => {
                let aspect = f64::from(size_in_pixels.width) / f64::from(size_in_pixels.height);
                Point {
                    re: DEFAULT_AXIS_LENGTH / zoom,
                    im: (DEFAULT_AXIS_LENGTH / zoom) / aspect,
                }
            }
        };
        let origin: Point = match location {
            Location::Origin(o) => o,
            Location::Centre(c) => c - 0.5 * axes,
        };
        let alg_spec = AlgorithmSpec::new_arc(algorithm, max_iter, colourer);
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            y_offset: None,
            alg_spec,
        }
    }
    /// Alternate constructor taking an offset
    #[must_use]
    pub fn new_subtile(
        origin: Point,
        axes: Point,
        size_in_pixels: Rect<u32>,
        // If present, this tile is part of a larger plot; this is its Pixel offset (width, height) within
        y_offset: u32,
        alg_spec: &Arc<AlgorithmSpec>,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            size_in_pixels,
            y_offset: Some(y_offset),
            alg_spec: Arc::clone(alg_spec),
        }
    }

    /// Splits this tile up into a number of strips, for parallel processing.
    /// The output vector is guaranteed to be output in Y offset order, starting from 0.
    pub fn split(&self, row_height: u32, debug: u8) -> anyhow::Result<Vec<TileSpec>> {
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
        let mut y_offset = self.height();

        let mut output = Vec::<TileSpec>::with_capacity(n_whole as usize + 1);
        for i in 0..n_whole {
            // Note we subtract the offset height before using it.
            // This has the property that after the last whole strip, height is either 0, or is the height of the remainder strip.
            y_offset -= row_height;
            output.push(TileSpec::new_subtile(
                working_origin,
                axes,
                strip_pixel_size,
                y_offset,
                &self.alg_spec,
            ));
            if debug > 0 {
                println!("tile {i} origin {working_origin} offset {y_offset}");
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
            ensure!(
                y_offset == last_height,
                "Unexpected remainder strip height ({}, expected {last_height}) - logic error?",
                y_offset
            );
            y_offset = 0;
            output.push(TileSpec::new_subtile(
                working_origin,
                last_axes,
                Rect::new(self.width(), last_height),
                y_offset,
                &self.alg_spec,
            ));
        }
        // Finally: We have worked from the bottom to the top. Reverse the order per our contract.
        output.reverse();
        Ok(output)
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
    /// Accessor
    #[must_use]
    pub fn algorithm(&self) -> &fractal::Instance {
        &self.alg_spec.algorithm
    }
    /// Accessor
    #[must_use]
    pub fn y_offset(&self) -> Option<u32> {
        self.y_offset
    }
    /// Accessor
    #[must_use]
    pub fn max_iter_requested(&self) -> u32 {
        self.alg_spec.max_iter
    }
    /// Accessor
    #[must_use]
    pub fn colourer(&self) -> &colouring::Colourer {
        &self.alg_spec.colourer
    }

    /// Computes the number of decimal significant figures needed to represent the axes for a given canvas size
    #[must_use]
    fn axes_precision_for_plot(canvas: Rect<u32>) -> usize {
        // Rationale: If a change in axes would move us <1 pixel it has no visible effect.
        #![allow(clippy::cast_lossless)] // cast to Scalar; we assume that pixel size will be precisely representable
        #![allow(clippy::cast_possible_truncation)] // cast to usize; we know that log(f64) fits into a u32
        #![allow(clippy::cast_sign_loss)] // cast to usize; we know that log(positive f64) is itself positive
        let max_dim = std::cmp::max(canvas.height, canvas.width) as Scalar;
        max_dim.log10().ceil() as usize
    }

    /// Computes the number of decimal significant figures needed to represent the axes of this tile
    #[must_use]
    pub fn axes_precision(&self) -> usize {
        TileSpec::axes_precision_for_plot(self.size_in_pixels)
    }

    fn format_origin(&self) -> String {
        use crate::util::float_format::DisplayDecimalPlacesTrimmed;
        let mut buf = String::new();
        let dp = self.position_decimal_places();
        self.origin.fmt_with_dp(&mut buf, dp).unwrap_or_default();
        buf
    }

    /// Computes the number of decimal places required for a given canvas and axes size
    #[must_use]
    fn position_decimal_places_for_plot(canvas: Rect<u32>, axes: Point) -> usize {
        #![allow(clippy::cast_lossless)] // casts to Scalar; we assume that pixel size will be precisely representable
        #![allow(clippy::cast_possible_truncation)] // cast to usize; we know that log(f64) fits into a u32
        #![allow(clippy::cast_sign_loss)] // cast to usize; we know that log(positive f64) is itself positive

        // Rationale: If a change in position would move us <1 pixel it has no visible effect.
        let pixel_size = Point::new(
            axes.re / canvas.width as Scalar,
            axes.im / canvas.height as Scalar,
        );
        let log_pixel = -f64::max(pixel_size.re, pixel_size.im).log10();
        log_pixel.ceil() as usize
    }

    /// Computes the number of decimal places needed to represent the position of this tile
    #[must_use]
    fn position_decimal_places(&self) -> usize {
        TileSpec::position_decimal_places_for_plot(self.size_in_pixels, self.axes)
    }

    fn format_axes(&self) -> String {
        use crate::util::float_format::DisplaySignificantFigures;
        let mut buf = String::new();
        self.axes
            .fmt_with_sf(&mut buf, self.axes_precision())
            .unwrap_or_default();
        buf
    }
}

impl Display for TileSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{},origin={},axes={},max={},col={}",
            self.alg_spec.algorithm,
            self.format_origin(),
            self.format_axes(),
            self.alg_spec.max_iter,
            self.alg_spec.colourer
        )
    }
}

impl std::cmp::PartialEq for TileSpec {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin     // False if both comparisands are NaN!
            && self.axes == other.axes  // False if both comparisands are NaN!
            && self.size_in_pixels == other.size_in_pixels
            && alg_specs_are_equivalent(&self.alg_spec, &other.alg_spec)
        // y_offset is irrelevant for equivalence
    }
}

// this is safe in our use case, see comment for TileSpec
impl std::cmp::Eq for TileSpec {}

/// A special Hash-like trait that may violate the standard rules
trait HairRaisingHash {
    fn hair_raising_hash<H: std::hash::Hasher>(&self, state: &mut H);
}

impl HairRaisingHash for AlgorithmSpec {
    fn hair_raising_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.algorithm.hash(state);
        self.max_iter.hash(state);
        // but NOT colourer, as the coloured output is not stored in a Tile.
    }
}

/// Hashing f64 is safe in our use case (see comments on ``TileSpec``).
impl HairRaisingHash for f64 {
    fn hair_raising_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl HairRaisingHash for Complex<f64> {
    fn hair_raising_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.re.hair_raising_hash(state);
        self.im.hair_raising_hash(state);
    }
}

// Specialist hash for caching
// this is safe in our use case, see comment for TileSpec
impl Hash for TileSpec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin.hair_raising_hash(state);
        self.axes.hair_raising_hash(state);
        self.size_in_pixels.hash(state);
        self.alg_spec.hair_raising_hash(state);
        // y_offset is irrelevant for cacheing
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hasher};

    use crate::{
        colouring::{
            self, Colourer,
            direct_rgb::{BlackFade, WhiteFade},
        },
        fractal::{self, Location, Point, Scalar, Size, TileSpec},
        util::Rect,
    };
    use approx::assert_relative_eq;

    const ZERO: Point = Point { re: 0.0, im: 0.0 };
    const ONE: Point = Point { re: 1.0, im: 1.0 };
    const ONETWO: Point = Point { re: 1.0, im: 2.0 };
    const CENTI: Point = Point { re: 0.01, im: 0.01 };

    const MANDELBROT: fractal::Instance =
        fractal::Instance::Original(fractal::mandelbrot::Original {});

    const BLACK_FADE: colouring::Colourer =
        colouring::Colourer::BlackFade(colouring::direct_rgb::BlackFade {});

    fn td_centre() -> TileSpec {
        TileSpec::new(
            Location::Centre(ONETWO),
            Size::AxesLength(ONE),
            // centre(1,2) => origin (0.5,1.5)
            Rect::<u32> {
                width: 100,
                height: 100,
            },
            MANDELBROT,
            256,
            BLACK_FADE,
        )
    }
    const TD_CENTRE_ORIGIN: Point = Point { re: 0.5, im: 1.5 };

    fn td_200h() -> TileSpec {
        TileSpec::new(
            Location::Centre(ZERO),
            Size::AxesLength(ONE),
            Rect::<u32> {
                width: 100,
                height: 200,
            },
            MANDELBROT,
            256,
            BLACK_FADE,
        )
    }

    #[test]
    fn origin_axes_pass_through() {
        let td = TileSpec::new(
            Location::Origin(ZERO),
            Size::AxesLength(ONE),
            Rect::<u32> {
                width: 100,
                height: 100,
            },
            MANDELBROT,
            256,
            BLACK_FADE,
        );
        assert_eq!(td.axes, ONE);
        assert_eq!(td.origin, ZERO);
    }

    #[test]
    fn pixel_size_divides() {
        let td = TileSpec::new(
            Location::Origin(ZERO),
            Size::PixelSize(CENTI),
            Rect::<u32> {
                width: 100,
                height: 100,
            },
            // this has the property that {width,height} * CENTI = { 1,1 }
            MANDELBROT,
            256,
            BLACK_FADE,
        );
        assert_eq!(td.axes, ONE);
    }
    #[test]
    fn aspect_axes() {
        const EXPECTED: Point = Point {
            re: 0.004,
            im: 0.002,
        };

        let td = TileSpec::new(
            Location::Origin(ZERO),
            Size::ZoomFactor(1000.0),
            Rect::<u32> {
                width: 200,
                height: 100,
            },
            // note funky aspect ratio.
            // 4.0 default axis * zoom factor 1000 = 0.004 across
            // 200x100 pixels => (0.004,0.002) axes.
            MANDELBROT,
            256,
            BLACK_FADE,
        );
        assert_eq!(td.axes, EXPECTED);
    }

    #[test]
    fn centre_computed() {
        assert_eq!(td_centre().origin, TD_CENTRE_ORIGIN);
    }
    #[test]
    fn top_left_computed() {
        // centre(1,2) & axes (1,1) => top-left (0.5,2.5)
        let expected = Point { re: 0.5, im: 2.5 };
        assert_eq!(td_centre().top_left(), expected);
    }
    #[test]
    fn bottom_right_computed() {
        // centre(1,2) & axes (1,1) => top-left (1.5,1.5)
        let expected = Point { re: 1.5, im: 1.5 };
        assert_eq!(td_centre().bottom_right(), expected);
    }

    #[test]
    fn split_strips_no_remainder() {
        const TEST_HEIGHT: u32 = 10;
        let spec = td_200h();
        assert_eq!(
            spec.height() % TEST_HEIGHT,
            0,
            "This test requires a test spec that is a multiple of {TEST_HEIGHT} pixels high"
        );
        let result = spec.split(TEST_HEIGHT, 0).unwrap();
        assert_eq!(
            result.len(),
            (spec.height() / TEST_HEIGHT) as usize,
            "Wrong number of output strips"
        );
        assert!(strips_in_order(&result));
        sanity_check_strips(&spec, &result, TEST_HEIGHT, None);
    }

    #[test]
    fn split_strips_with_remainder() {
        const TEST_HEIGHT: u32 = 11;
        let spec = td_200h();
        let remainder = spec.height() % TEST_HEIGHT;
        assert_ne!(
            remainder, 0,
            "This test requires a test spec that is not a multiple of {TEST_HEIGHT} pixels high"
        );
        let result = spec.split(TEST_HEIGHT, 0).unwrap();
        assert_eq!(
            result.len(),
            1 + (spec.height() / TEST_HEIGHT) as usize,
            "Wrong number of output strips"
        );
        assert!(strips_in_order(&result));
        sanity_check_strips(&spec, &result, TEST_HEIGHT, Some(remainder));
    }

    fn strips_in_order(strips: &[TileSpec]) -> bool {
        strips.windows(2).all(|w| w[0].y_offset < w[1].y_offset)
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
            let offset = ts.y_offset().unwrap();
            assert!(offset <= spec.height());
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
            Location::Origin(Point { re: -2.0, im: -2.0 }),
            Size::AxesLength(Point { re: 4.0, im: 4.0 }),
            Rect::new(100, 100),
            MANDELBROT,
            256,
            BLACK_FADE,
        );
        assert!(ts.auto_adjust_aspect_ratio().is_ok_and(|v| v.is_none()));
    }

    #[test]
    fn aspect_gt_1() {
        let ts = TileSpec::new(
            Location::Origin(Point { re: -2.0, im: -2.0 }),
            Size::AxesLength(Point { re: 4.0, im: 4.0 }),
            Rect::new(200, 100),
            MANDELBROT,
            256,
            BLACK_FADE,
        );
        check_aspect(ts);
    }
    #[test]
    fn aspect_lt_1() {
        let ts = TileSpec::new(
            Location::Origin(Point { re: -2.0, im: -2.0 }),
            Size::AxesLength(Point { re: 4.0, im: 4.0 }),
            Rect::new(100, 200),
            MANDELBROT,
            256,
            BLACK_FADE,
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
            Location::Origin(Point { re: 0.0, im: -0.5 }),
            Size::AxesLength(Point { re: -1.0, im: 2.0 }),
            Rect::new(200, 400),
            fractal::framework::factory(fractal::framework::Selection::Original),
            256,
            BLACK_FADE,
        );
        let result = uut.to_string();
        assert_eq!(
            result,
            "original,origin=0-0.5i,axes=-1+2i,max=256,col=black-fade"
        );
    }

    fn hash<T: std::hash::Hash>(it: &T) -> u64 {
        let mut h = DefaultHasher::new();
        it.hash(&mut h);
        h.finish()
    }

    #[test]
    fn hashability() {
        fn test_tilespec(f: fractal::framework::Selection) -> TileSpec {
            TileSpec::new(
                Location::Origin(Point { re: 0.0, im: -0.5 }),
                Size::AxesLength(Point { re: -1.0, im: 2.0 }),
                Rect::new(200, 400),
                fractal::framework::factory(f),
                256,
                BLACK_FADE,
            )
        }

        let uut = test_tilespec(fractal::framework::Selection::Original);
        let h1 = hash(&uut);
        let h2 = hash(&uut);
        assert_eq!(h1, h2);

        let uut2 = test_tilespec(fractal::framework::Selection::Mandel3);
        let h3 = hash(&uut2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn different_colourer_same_hash() {
        fn test_tilespec(c: Colourer) -> TileSpec {
            TileSpec::new(
                Location::Origin(Point { re: 0.0, im: -0.5 }),
                Size::AxesLength(Point { re: -1.0, im: 2.0 }),
                Rect::new(200, 400),
                MANDELBROT,
                256,
                c,
            )
        }
        let data1 = test_tilespec(BlackFade {}.into());
        let h1 = hash(&data1);
        let data2 = test_tilespec(WhiteFade {}.into());
        let h2 = hash(&data2);
        assert_eq!(h1, h2);
    }
}
