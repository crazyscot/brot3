// Fractal algorithm selection, dispatch framework & shared code
// (c) 2024 Ross Younger

use super::{Point, PointData, ESCAPE_THRESHOLD_SQ};

use super::mandelbrot::{Mandel3, Original};
use super::mandeldrop::{Mandeldrop, Mandeldrop3};

use enum_dispatch::enum_dispatch;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString};

/// Selector for available Algorithms
#[enum_dispatch]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumMessage, PartialEq)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(name(Selection), derive(clap::ValueEnum, EnumIter, EnumString))] // ... and specifies what it derives from
#[allow(clippy::module_name_repetitions)] // enum_dispatch doesn't support structs with the same name but different paths
pub enum FractalInstance {
    /// The original Mandelbrot set, z := z^2+c (aliases: "m", "m2")
    #[strum_discriminants(value(alias = "m", alias = "m2"))]
    Original,
    /// Mandelbrot^3 z:=z^3+c (alias: "m3")
    #[strum_discriminants(value(alias = "m3"))]
    Mandel3,

    #[strum_discriminants(value(alias = "drop"))]
    /// Mandeldrop (Inverted set) z:=z^2+c using 1/z0 (alias: drop)
    Mandeldrop,

    #[strum_discriminants(value(alias = "drop3"))]
    /// Mandeldrop (Inverted set) z:=z^3+c using 1/z0 (alias: drop3)
    Mandeldrop3,

    /// Test algorithm that always outputs zero
    #[strum(disabled)]
    Zero,
}

/// Factory method for fractals
#[must_use]
pub fn factory(selection: Selection) -> FractalInstance {
    match selection {
        Selection::Original => Original::default().into(),
        Selection::Mandel3 => Mandel3::default().into(),
        Selection::Mandeldrop => Mandeldrop::default().into(),
        Selection::Mandeldrop3 => Mandeldrop3::default().into(),
        Selection::Zero => Zero {}.into(),
    }
}

/// A fractal algorithm
/// This knows nothing about colouring, only maths on the complex plane.
#[enum_dispatch(FractalInstance)]
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
            if point.value.norm_sqr() >= ESCAPE_THRESHOLD_SQ {
                self.finish(point);
                return;
            }
        }
    }
    /// Finalises the point data once a pixel has escaped
    fn finish(&self, point: &mut PointData);

    /// The default plot origin for this fractal
    fn default_centre(&self) -> Point {
        Point { re: 0.0, im: 0.0 }
    }
    /// The default plot axes for this fractal
    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

/// Test algorithm, doesn't do anything useful
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Zero {}
impl Algorithm for Zero {
    fn iterate(&self, point: &mut PointData) {
        point.result = Some(0.0);
    }
    fn finish(&self, _point: &mut PointData) {}
}

#[cfg(test)]
mod tests {
    use super::FractalInstance;
    use crate::util::listable::list_vec;

    #[test]
    fn renderers_list() {
        assert_ne!(list_vec::<FractalInstance>().len(), 0);
    }
}
