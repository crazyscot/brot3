// Specification of a plot, human-friendly form
// (c) 2024 Ross Younger

use super::{Point, Scalar};

/// The user is allowed to specify the plot location in multiple ways.
#[derive(Debug, Clone, Copy)]
pub enum Location {
    /// The origin point (bottom-left corner i.e. smallest real,imaginary coefficients)
    Origin(Point),
    /// The centre point
    Centre(Point),
}

/// The user is allowed to specify the plot size in multiple ways.
#[derive(Debug, Clone, Copy)]
pub enum Size {
    /// Length of both axes
    AxesLength(Point),
    /// Size of a pixel in both dimensions
    PixelSize(Point),
    /// Singular zoom factor on the Real axis (square pixels)
    ZoomFactor(Scalar),
    // TODO RealLength, RealPixel ?
}

/// User-friendly way to specify a plot
#[derive(Debug, Clone, Copy)]
pub struct PlotSpec {
    /// Location of the plot
    pub location: Location,
    /// Size of the plot
    pub axes: Size,
    /// Height in pixels
    pub height: u32,
    /// Width in pixels
    pub width: u32,
}

impl PlotSpec {
    /// Calculates the aspect ratio of the plot
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }
}
