// Specification of a plot, human-friendly form
// (c) 2024 Ross Younger

use super::{Point, Scalar};

/// The user is allowed to specify the plot location in multiple ways.
#[derive(Debug, Clone, Copy)]
pub enum UserPlotLocation {
    /// The origin (top-left corner) point
    Origin(Point),
    /// The centre point
    Centre(Point),
}

/// The user is allowed to specify the plot size in multiple ways.
#[derive(Debug, Clone, Copy)]
pub enum UserPlotSize {
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
pub struct UserPlotSpec {
    /// Location of the plot
    pub location: UserPlotLocation,
    /// Size of the plot
    pub axes: UserPlotSize,
    /// Height in pixels
    pub height: u32,
    /// Width in pixels
    pub width: u32,
}

impl UserPlotSpec {
    /// Calculates the aspect ratio of the plot
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}
