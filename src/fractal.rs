// Fractal plotting
// (c) 2024 Ross Younger

mod mandelbrot;
mod pointdata;
mod tile;
mod tilespec;
mod userplotspec;

pub use mandelbrot::{mandelbrot_iterate, mandelbrot_pixel, mandelbrot_prepare}; // TEMP
pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::TileSpec;
pub use userplotspec::{Location, PlotSpec, Size};

use enum_dispatch::enum_dispatch;
use mandelbrot::Original;
use num_complex::Complex;
use strum::{EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString};

/// One dimension of a fractal
pub type Scalar = f64;
/// ln(2) for the Scalar type
const SCALAR_LN_2: Scalar = std::f64::consts::LN_2;
/// Complex type for fractals
pub type Point = Complex<Scalar>;

/// Selector for available Algorithms
#[enum_dispatch]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumMessage)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum AlgorithmEnumDiscriminants ...
#[strum_discriminants(derive(clap::ValueEnum, EnumIter, EnumString))] // ... and specifies what it derives from
pub enum Selection {
    /// The original Mandelbrot set (aliases: "m", "m2")
    #[strum_discriminants(value(alias = "m", alias = "m2"))]
    Original,
}

const ESCAPE_THRESHOLD: Scalar = 4.0;

/// A fractal algorithm
/// This knows nothing about colouring, only maths on the complex plane.
#[enum_dispatch(Algorithm)]
pub trait Algorithm {
    /// Prepares the ``PointData`` to iterate
    fn prepare(&self, point: &mut PointData);
    /// The iteration function
    fn iterate(&self, point: &mut PointData);
    /// Runs the iteration for a single point, up to a given limit.
    /// The default implementation is expected to suit most algorithms.
    fn pixel(&self, point: &mut PointData, max_iter: u32) {
        for _ in point.iter..max_iter {
            self.iterate(point);
            if point.value.norm_sqr() >= ESCAPE_THRESHOLD {
                self.finish(point);
                return;
            }
        }
    }
    /// Finalises the point data once a pixel has escaped
    fn finish(&self, point: &mut PointData);
}

/// Lists all available renderers
#[must_use]
pub fn list_vec() -> Vec<String> {
    Selection::iter().map(|a| a.to_string()).collect()
}

/// Implementation of 'list fractals'
pub fn list(machine_parseable: bool) {
    if machine_parseable {
        println!("{:?}", list_vec());
        return;
    }

    println!("Available fractals:");
    let longest = Selection::iter()
        .map(|r| r.to_string().len())
        .max()
        .unwrap_or(1);

    let _ = Selection::iter()
        .map(|r| {
            println!(
                "  {:width$}  {}",
                r.to_string(),
                r.get_documentation().unwrap_or_default(),
                width = longest
            );
        })
        .collect::<Vec<_>>();
}

/// Factory method for fractals
#[must_use]
pub fn factory(selection: SelectionDiscriminants) -> Selection {
    match selection {
        SelectionDiscriminants::Original => Selection::Original(mandelbrot::Original {}),
    }
}

#[cfg(test)]
mod tests {
    use super::list_vec;

    #[test]
    fn renderers_list() {
        assert_ne!(list_vec().len(), 0);
    }
}
