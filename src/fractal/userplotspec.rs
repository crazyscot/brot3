// Data about the plot, human-friendly form
// (c) 2024 Ross Younger

use super::{Point, Scalar};

/// The user is allowed to specify the plot location in multiple ways.
#[derive(Debug, Clone)]
pub enum UserPlotLocation {
    Origin(Point),
    Centre(Point),
}

/// The user is allowed to specify the plot size in multiple ways.
#[derive(Debug, Clone)]
pub enum UserPlotSize {
    /// Length of both axes
    AxesLength(Point),
    /// Size of a pixel in both dimensions
    PixelSize(Point),
    // Singular zoom factor on the Real axis (square pixels)
    ZoomFactor(Scalar),
    // TODO RealLength, RealPixel ?
}

/// User-friendly way to specify a plot
#[derive(Debug, Clone)]
pub struct UserPlotSpec {
    pub location: UserPlotLocation,
    pub axes: UserPlotSize,
    pub height: u32,
    pub width: u32,
}

impl UserPlotSpec {
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

// TODO: convert from a PlotData to the variants...
