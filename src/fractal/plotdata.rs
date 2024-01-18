// Specification of a plot (origin, axes, etc)
// (c) 2024 Ross Younger

use super::{Point, Scalar, Tile};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct PlotData {
    pub origin: Point,
    pub axes: Point,
}

impl PlotData {
    // TODO allow creation by origin/centre, axes length/pixel size
    pub fn pixel_size(&self, tile: &Tile) -> Point {
        Point {
            re: self.axes.re / tile.width as Scalar,
            im: self.axes.im / tile.height as Scalar,
        }
    }
    // XXX From (UserPlotData)
}

impl Display for PlotData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "@{} axes={}", self.origin, self.axes)
    }
}
