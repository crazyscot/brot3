// Fractal plotting
// (c) 2024 Ross Younger

mod framework;
mod mandelbrot;
mod pointdata;
mod tile;
mod tilespec;
mod userplotspec;

#[allow(clippy::module_name_repetitions)]
pub use framework::{factory, Algorithm, FractalInstance, Selection};
pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::{SplitMethod, TileSpec};
pub use userplotspec::{Location, PlotSpec, Size};

use num_complex::Complex;

/// One dimension of a fractal
pub type Scalar = f64;
/// ln(2) for the Scalar type
const SCALAR_LN_2: Scalar = std::f64::consts::LN_2;
/// Complex type for fractals
pub type Point = Complex<Scalar>;

/// The square of the standard escape-time threshold
pub const ESCAPE_THRESHOLD_SQ: Scalar = 4.0;
