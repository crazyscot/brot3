// Specification of a plot, human-friendly form
// (c) 2024 Ross Younger

use super::{Point, Scalar};
use crate::{colouring, fractal, util::Rect};

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
    /// Size of the plot on the complex plane
    pub axes: Size,
    /// Size of the plot in pixels
    pub size_in_pixels: Rect<u32>,
    /// The selected algorithm
    pub algorithm: fractal::Instance,
    /// The iteration limit
    pub max_iter: u32,
    /// The selected colourer
    pub colourer: colouring::Instance,
}

impl PlotSpec {
    /// Calculates the aspect ratio of the plot
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.size_in_pixels.width) / f64::from(self.size_in_pixels.height)
    }

    /// Accessor
    #[must_use]
    pub fn width(&self) -> u32 {
        self.size_in_pixels.width
    }
    /// Accessor
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size_in_pixels.height
    }
}
