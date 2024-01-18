// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use super::userplotdata::{UserPlotLocation, UserPlotSize};
use super::{Point, Scalar, Tile, UserPlotData};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct PlotData {
    pub origin: Point,
    pub axes: Point,
}

/// Canonicalised data about a plot.
/// For convenient construction, use From<&UserPlotData>.
impl PlotData {
    pub fn pixel_size(&self, tile: &Tile) -> Point {
        Point {
            re: self.axes.re / tile.width as Scalar,
            im: self.axes.im / tile.height as Scalar,
        }
    }
}

const DEFAULT_AXIS_LENGTH: Scalar = 4.0;

impl From<&UserPlotData> for PlotData {
    fn from(upd: &UserPlotData) -> Self {
        // Must compute axes first as origin may depend on them
        let axes: Point = match upd.axes {
            UserPlotSize::AxesLength(l) => l,
            UserPlotSize::PixelSize(p) => Point {
                re: p.re * upd.width as Scalar,
                im: p.im * upd.height as Scalar,
            },
            UserPlotSize::ZoomFactor(zoom) => Point {
                re: DEFAULT_AXIS_LENGTH / zoom,
                im: (DEFAULT_AXIS_LENGTH / zoom) / upd.aspect_ratio(),
            },
        };
        let origin: Point = match upd.location {
            UserPlotLocation::Origin(o) => o,
            UserPlotLocation::Centre(c) => c - 0.5 * axes,
        };
        PlotData { origin, axes }
    }
}

impl Display for PlotData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "origin={} axes={}", self.origin, self.axes)
    }
}

#[cfg(test)]
mod tests {
    use crate::fractal::{
        userplotdata::{UserPlotLocation, UserPlotSize},
        PlotData, Point, UserPlotData,
    };

    const ZERO: Point = Point { re: 0.0, im: 0.0 };
    const ONE: Point = Point { re: 1.0, im: 1.0 };
    const ONETWO: Point = Point { re: 1.0, im: 2.0 };
    const CENTI: Point = Point { re: 0.01, im: 0.01 };

    const TD_ORIGIN_AXES: UserPlotData = UserPlotData {
        location: UserPlotLocation::Origin(ZERO),
        axes: UserPlotSize::AxesLength(ONE),
        height: 100,
        width: 100,
    };
    const TD_ORIGIN_PIXELS: UserPlotData = UserPlotData {
        location: UserPlotLocation::Origin(ZERO),
        axes: UserPlotSize::PixelSize(CENTI),
        height: 100,
        width: 100,
        // this has the property that {width,height} * CENTI = { 1,1 }
    };
    const TD_ORIGIN_ZOOM: UserPlotData = UserPlotData {
        location: UserPlotLocation::Origin(ZERO),
        axes: UserPlotSize::ZoomFactor(1000.0),
        height: 100,
        width: 200,
        // note funky aspect ratio.
        // 4.0 default axis * zoom factor 1000 = 0.004 across
        // 200x100 pixels => (0.004,0.002) axes.
    };
    const TD_CENTRE: UserPlotData = UserPlotData {
        location: UserPlotLocation::Centre(ONETWO),
        axes: UserPlotSize::AxesLength(ONE),
        // centre(1,2) => origin (0.5,1.5)
        height: 100,
        width: 100,
    };

    const TD_ORIGIN_ZOOM_AXES: Point = Point {
        re: 0.004,
        im: 0.002,
    };
    const TD_CENTRE_ORIGIN: Point = Point { re: 0.5, im: 1.5 };

    #[test]
    fn axes_pass_through() {
        let result = PlotData::from(&TD_ORIGIN_AXES);
        assert_eq!(result.axes, ONE);
    }
    #[test]
    fn pixel_size_divides() {
        let result = PlotData::from(&TD_ORIGIN_PIXELS);
        assert_eq!(result.axes, ONE);
    }
    #[test]
    fn aspect_axes() {
        assert_eq!(TD_ORIGIN_ZOOM.aspect_ratio(), 2.0);
        let result = PlotData::from(&TD_ORIGIN_ZOOM);
        assert_eq!(result.axes, TD_ORIGIN_ZOOM_AXES);
    }

    #[test]
    fn origin_pass_through() {
        let result = PlotData::from(&TD_ORIGIN_AXES);
        assert_eq!(result.origin, ZERO);
    }
    #[test]
    fn centre_computed() {
        let result = PlotData::from(&TD_CENTRE);
        assert_eq!(result.origin, TD_CENTRE_ORIGIN);
    }
}
