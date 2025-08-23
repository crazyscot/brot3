// Fractal algorithm selection, dispatch framework & shared code
// (c) 2024 Ross Younger

#![allow(missing_docs)] // EnumDiscriminants

use std::str::FromStr;

use super::{ESCAPE_THRESHOLD_SQ, Point, PointData};

use super::mandelbrot::{Mandel3, Original};
use super::mandeldrop::{Mandeldrop, Mandeldrop3};
use super::misc_fractals::{BirdOfPrey, Buffalo, BurningShip, Celtic, Mandelbar, Variant};

use enum_delegate;
use strum::IntoStaticStr;
use strum_macros::{
    Display, EnumDiscriminants, EnumMessage, EnumProperty, EnumString, FromRepr, VariantArray,
};

/// Selector for available Algorithms
#[enum_delegate::implement(Algorithm)]
#[derive(
    Clone, Copy, Debug, Display, FromRepr, PartialEq, Eq, Hash, PartialOrd, Ord, IntoStaticStr,
)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(
    name(Selection),
    derive(
        clap::ValueEnum,
        Display,
        EnumMessage,
        EnumProperty,
        EnumString,
        VariantArray,
        PartialOrd,
        Ord,
    )
)] // ... and specifies what it derives from
pub enum Instance {
    /// The original Mandelbrot set, `z := z^2+c` (aliases: "m", "m2")
    #[strum_discriminants(value(alias = "m", alias = "m2"))]
    Original(Original),
    /// Mandelbrot^3 z:=z^3+c (alias: "m3")
    #[strum_discriminants(value(alias = "m3"))]
    Mandel3(Mandel3),

    #[strum_discriminants(value(alias = "drop"))]
    /// Mandeldrop (Inverted set) `z:=z^2+c` using 1/z0 (alias: drop)
    Mandeldrop(Mandeldrop),

    #[strum_discriminants(value(alias = "drop3"))]
    /// Mandeldrop (Inverted set) `z:=z^3+c` using 1/z0 (alias: drop3)
    Mandeldrop3(Mandeldrop3),

    #[strum_discriminants(value(alias = "bar"))]
    /// Mandelbar (Tricorn) `z:=(z*)^2+c` (alias: bar)
    Mandelbar(Mandelbar),

    #[strum_discriminants(value(alias = "ship"))]
    /// The Burning Ship `z:=(|Re(z)|+i|Im(z)|)^2+c` (alias: ship)
    BurningShip(BurningShip),

    /// The Generalised Celtic `z:= (|Re(z^2)| + i.Im(z^2) + c)`
    Celtic(Celtic),

    /// The Variant `z:=z^2+c with Re(z):=|Re(z)|` on odd iterations
    Variant(Variant),

    #[strum_discriminants(value(alias = "bird"))]
    /// Bird of Prey `z:=(Re(z)+i|Im(z)|)^2+c` (alias: bird)
    BirdOfPrey(BirdOfPrey),

    /// Buffalo `z:=|z|^2 - |z| + c`
    Buffalo(Buffalo),

    /// Test algorithm that always outputs zero
    #[strum_discriminants(strum(props(hide_from_list = "1")))]
    Zero(Zero),
}

impl Default for Selection {
    fn default() -> Self {
        Self::Original
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::Original(Original {})
    }
}

impl crate::util::listable::Listable for Selection {}

/// Factory method for fractals
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn factory(selection: Selection) -> Instance {
    Instance::from_repr(selection as usize).unwrap_or_else(|| {
        panic!("Failed to convert enum discriminant {selection} into instance (can't happen)")
    })
}

impl FromStr for Instance {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match Selection::from_str(s) {
            Ok(s) => Ok(factory(s)),
            Err(_) => anyhow::bail!("unknown fractal name"),
        }
    }
}

/// A fractal algorithm
/// This knows nothing about colouring, only maths on the complex plane.
#[enum_delegate::register]
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

    /// The default plot centre for this fractal
    fn default_centre(&self) -> Point {
        Point { re: 0.0, im: 0.0 }
    }
    /// The default plot axes for this fractal
    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

/// Test algorithm, doesn't do anything useful
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Zero {}
impl Algorithm for Zero {
    fn iterate(&self, point: &mut PointData) {
        point.result = Some(0.0);
    }
    fn finish(&self, _point: &mut PointData) {}
}
