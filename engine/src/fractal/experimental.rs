// Experimental fractals
// (c) 2024 Ross Younger

use super::mandelbrot::Original;
use super::maths::Point;
use super::{Algorithm, PointData};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComplexPowerExperiment {
    delegate: Original,
}

impl Algorithm for ComplexPowerExperiment {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        // The first iteration is easy
        point.value = point.origin;
        point.iter = 1;
    }

    #[inline]
    fn iterate(&self, point: &mut PointData) {
        point.value = point.value.powc(point.value) + point.origin;
        point.iter += 1;
    }

    #[doc = r"Finalises the point data once a pixel has escaped"]
    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: 0.0, im: 0.0 }
    }
    fn default_axes(&self) -> super::Point {
        Point { re: 6.0, im: 6.0 }
    }
}
