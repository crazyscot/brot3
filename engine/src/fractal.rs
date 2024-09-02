// Fractal plotting
// (c) 2024 Ross Younger

/// Maths type selection and constants
pub mod maths;

mod framework;
mod mandelbrot;
mod mandeldrop;
mod misc_fractals;
mod pointdata;
mod tile;
mod tilespec;

#[allow(clippy::module_name_repetitions)]
pub use framework::{decode, factory, Algorithm, Instance, Selection};
pub use maths::{Point, Scalar};
pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::TileSpec;

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
}

/// The square of the standard escape-time threshold
pub const ESCAPE_THRESHOLD_SQ: Scalar = 4.0;
