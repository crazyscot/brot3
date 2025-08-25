// Fractal algorithm selection, dispatch framework & shared code
// (c) 2024 Ross Younger

use super::{ESCAPE_THRESHOLD_SQ, Point, PointData};

use super::mandelbrot::{Mandel3, Original};
use super::mandeldrop::{Mandeldrop, Mandeldrop3};
use super::misc_fractals::{BirdOfPrey, Buffalo, BurningShip, Celtic, Mandelbar, Variant};

use spire_enum::prelude::{delegate_impl, delegated_enum};

/// Framework for all available fractal algorithms.
/// see [`IAlgorithm`]
#[delegated_enum(impl_conversions)]
#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum_macros::FromRepr,
    strum_macros::Display,
    strum_macros::EnumDiscriminants,
    strum_macros::EnumIter,
    strum_macros::EnumMessage,
    strum_macros::EnumProperty,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "kebab_case")]
pub enum Algorithm {
    /// The original Mandelbrot set, `z := z^2+c` (aliases: "m", "m2")
    #[strum(
        serialize = "m",
        serialize = "m2",
        serialize = "mandelbrot",
        serialize = "original"
    )]
    Original(Original),
    /// Mandelbrot^3 z:=z^3+c (alias: "m3")
    #[strum(serialize = "m3", serialize = "mandel3")]
    Mandel3(Mandel3),

    #[strum(serialize = "drop", serialize = "mandeldrop")]
    /// Mandeldrop (Inverted set) `z:=z^2+c` using 1/z0 (alias: drop)
    Mandeldrop(Mandeldrop),

    #[strum(serialize = "drop3", serialize = "mandeldrop3")]
    /// Mandeldrop (Inverted set) `z:=z^3+c` using 1/z0 (alias: drop3)
    Mandeldrop3(Mandeldrop3),

    #[strum(serialize = "bar", serialize = "mandelbar")]
    /// Mandelbar (Tricorn) `z:=(z*)^2+c` (alias: bar)
    Mandelbar(Mandelbar),

    #[strum(serialize = "ship", serialize = "burningship")]
    /// The Burning Ship `z:=(|Re(z)|+i|Im(z)|)^2+c` (alias: ship)
    BurningShip(BurningShip),

    /// The Generalised Celtic `z:= (|Re(z^2)| + i.Im(z^2) + c)`
    Celtic(Celtic),

    /// The Variant `z:=z^2+c with Re(z):=|Re(z)|` on odd iterations
    Variant(Variant),

    #[strum(serialize = "bird", serialize = "bird-of-prey")]
    /// Bird of Prey `z:=(Re(z)+i|Im(z)|)^2+c` (alias: bird)
    BirdOfPrey(BirdOfPrey),

    /// Buffalo `z:=|z|^2 - |z| + c`
    Buffalo(Buffalo),

    /// Test algorithm that always outputs zero
    #[strum(props(hide_from_list = "1"))]
    Zero(Zero),
}

impl crate::util::Listable for Algorithm {}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::Original(Original {})
    }
}

/// A fractal algorithm
/// This knows nothing about colouring, only maths on the complex plane.
pub trait IAlgorithm {
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

    /// The default plot centre for this fractal
    fn default_centre(&self) -> Point {
        Point { re: 0.0, im: 0.0 }
    }
    /// The default plot axes for this fractal
    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

#[delegate_impl]
impl IAlgorithm for Algorithm {
    fn prepare(&self, point: &mut PointData);
    fn iterate(&self, point: &mut PointData);
    #[inline]
    fn pixel(&self, point: &mut PointData, max_iter: u32);
    fn finish(&self, point: &mut PointData);
    fn default_centre(&self) -> Point;
    fn default_axes(&self) -> Point;
}

/// Test algorithm, doesn't do anything useful
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Zero {}
impl IAlgorithm for Zero {
    fn iterate(&self, point: &mut PointData) {
        point.result = Some(0.0);
    }
    fn finish(&self, _point: &mut PointData) {}
}
