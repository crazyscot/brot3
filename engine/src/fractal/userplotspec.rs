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
