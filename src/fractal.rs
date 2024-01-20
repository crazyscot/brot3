// Fractal plotting
// (c) 2024 Ross Younger

mod mandelbrot;
mod pointdata;
mod tile;
mod tilespec;
mod userplotspec;

use num_complex::Complex;

pub use mandelbrot::{mandelbrot_iterate, mandelbrot_pixel, mandelbrot_prepare}; // TEMP
pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::TileSpec;
pub use userplotspec::{Location, PlotSpec, Size};

/// One dimension of a fractal
pub type Scalar = f64;
/// ln(2) for the Scalar type
const SCALAR_LN_2: Scalar = std::f64::consts::LN_2;
/// Complex type for fractals
pub type Point = Complex<Scalar>;
