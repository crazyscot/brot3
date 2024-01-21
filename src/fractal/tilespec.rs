// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use super::userplotspec::{Location, Size};
use super::{FractalInstance, PlotSpec, Point, Scalar};

use std::fmt::{self, Display, Formatter};

/// Machine-facing specification of a tile to plot
#[derive(Debug, Clone, Copy)]
pub struct TileSpec {
    /// Plot origin (bottom-left corner, smallest real/imaginary coefficients)
    origin: Point,
    /// Plot axes length
    axes: Point,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,

    /// The selected algorithm
    algorithm: FractalInstance,
}

/// Canonicalised data about a plot.
/// For convenient construction, use From<&``UserPlotData``>.
impl TileSpec {
    /// Computes the pixel size for this spec.
    #[must_use]
    pub fn pixel_size(&self) -> Point {
        Point {
            re: self.axes.re / Scalar::from(self.width),
            im: self.axes.im / Scalar::from(self.height),
        }
    }

    /// Constructor
    #[must_use]
    pub fn new(
        origin: Point,
        axes: Point,
        height: u32,
        width: u32,
        algorithm: FractalInstance,
    ) -> TileSpec {
        TileSpec {
            origin,
            axes,
            width,
            height,
            algorithm,
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
        self.height
    }
    /// Accessor
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Accessor
    #[must_use]
    pub fn algorithm(&self) -> FractalInstance {
        self.algorithm
    }
}

const DEFAULT_AXIS_LENGTH: Scalar = 4.0;

impl From<&PlotSpec> for TileSpec {
    fn from(upd: &PlotSpec) -> Self {
        // Must compute axes first as origin may depend on them
        let axes: Point = match upd.axes {
            Size::AxesLength(l) => l,
            Size::PixelSize(p) => Point {
                re: p.re * Scalar::from(upd.width),
                im: p.im * Scalar::from(upd.height),
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
            height: upd.height,
            width: upd.width,
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
    use crate::fractal::{
        self,
        userplotspec::{Location, Size},
        FractalInstance, PlotSpec, Point, TileSpec,
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
        height: 100,
        width: 100,
        algorithm: MANDELBROT,
    };
    const TD_ORIGIN_PIXELS: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::PixelSize(CENTI),
        height: 100,
        width: 100,
        // this has the property that {width,height} * CENTI = { 1,1 }
        algorithm: MANDELBROT,
    };
    const TD_ORIGIN_ZOOM: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::ZoomFactor(1000.0),
        height: 100,
        width: 200,
        // note funky aspect ratio.
        // 4.0 default axis * zoom factor 1000 = 0.004 across
        // 200x100 pixels => (0.004,0.002) axes.
        algorithm: MANDELBROT,
    };
    const TD_CENTRE: PlotSpec = PlotSpec {
        location: Location::Centre(ONETWO),
        axes: Size::AxesLength(ONE),
        // centre(1,2) => origin (0.5,1.5)
        height: 100,
        width: 100,
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
}
