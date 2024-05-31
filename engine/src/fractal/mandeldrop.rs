// Inverted Mandelbrot fractal
// (c) 2024 Ross Younger

use super::mandelbrot::{Mandel3, Original};
use super::maths::Point;
use super::{Algorithm, PointData};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mandeldrop {
    delegate: Original,
}

// Shared prep function for all Mandeldrops
#[inline]
fn prepare_drop(point: &mut PointData) {
    // This is the difference from the standard Mandelbrot.
    // The point is inverted i.e. we map the point z0 to 1/z0 and use that in place of the origin.
    let inv = point.origin.inv();
    point.origin = inv;
    point.value = inv;
    point.iter = 1;
}

impl Algorithm for Mandeldrop {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        prepare_drop(point);
    }
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        self.delegate.iterate(point);
    }
    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    fn default_centre(&self) -> Point {
        Point { re: 1.25, im: 0.0 }
    }

    fn default_axes(&self) -> Point {
        Point { re: 8.0, im: 8.0 }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mandeldrop3 {
    delegate: Mandel3,
}

impl Algorithm for Mandeldrop3 {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        prepare_drop(point);
    }
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        self.delegate.iterate(point);
    }
    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    // This fractal uses standard centre but wide axes
    fn default_axes(&self) -> Point {
        Point { re: 6.0, im: 6.0 }
    }
}
