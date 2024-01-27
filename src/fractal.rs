// Fractal plotting
// (c) 2024 Ross Younger

/// Maths type selection and constants
pub mod maths;

mod framework;
mod mandelbrot;
mod mandeldrop;
mod pointdata;
mod tile;
mod tilespec;
mod userplotspec;

#[allow(clippy::module_name_repetitions)]
pub use framework::{factory, Algorithm, FractalInstance, Selection};
pub use maths::{Point, Scalar};
pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::{SplitMethod, TileSpec};
pub use userplotspec::{Location, PlotSpec, Size};

/// The square of the standard escape-time threshold
pub const ESCAPE_THRESHOLD_SQ: Scalar = 4.0;
