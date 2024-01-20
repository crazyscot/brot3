// Fractal plotting
// (c) 2024 Ross Younger

mod mandelbrot;
mod pointdata;
mod tile;
mod tilespec;
mod userplotspec;

pub use pointdata::PointData;
pub use tile::Tile;
pub use tilespec::TileSpec;
pub use userplotspec::{Location, PlotSpec, Size};

use enum_dispatch::enum_dispatch;
use mandelbrot::{Mandel3, Original};
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
pub enum SelectionF {
    /// The original Mandelbrot set, z := z^2+c (aliases: "m", "m2")
    #[strum_discriminants(value(alias = "m", alias = "m2"))]
    Original,
    /// Mandelbrot^3 z:=z^3+c (alias: "m3")
    #[strum_discriminants(value(alias = "m3"))]
    Mandel3,
}

const ESCAPE_THRESHOLD: Scalar = 4.0;

/// A fractal algorithm
/// This knows nothing about colouring, only maths on the complex plane.
#[enum_dispatch(SelectionF)]
pub trait Algorithm {
    /// Algorithm-specific data preparation before we iterate for the first time
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        // This default is a reasonable optimisation for many fractals but may not be appropriate for all.
        // Some fractals may use this default and add additional tasks.
        point.value = point.origin;
        point.iter = 1;
    }
    /// The iteration function
    fn iterate(&self, point: &mut PointData);
    /// Runs the iteration for a single point, up to a given limit.
    /// The default implementation is expected to suit most algorithms.
    #[inline]
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
    SelectionF::iter().map(|a| a.to_string()).collect()
}

/// Implementation of 'list fractals'
pub fn list(machine_parseable: bool) {
    if machine_parseable {
        println!("{:?}", list_vec());
        return;
    }

    println!("Available fractals:");
    let longest = SelectionF::iter()
        .map(|r| r.to_string().len())
        .max()
        .unwrap_or(1);

    let _ = SelectionF::iter()
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
pub fn factory(selection: SelectionFDiscriminants) -> SelectionF {
    match selection {
        SelectionFDiscriminants::Original => SelectionF::Original(mandelbrot::Original {}),
        SelectionFDiscriminants::Mandel3 => SelectionF::Mandel3(mandelbrot::Mandel3 {}),
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
