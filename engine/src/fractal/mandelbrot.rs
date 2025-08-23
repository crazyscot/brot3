// Mandelbrot set implementation
// (c) 2024 Ross Younger

use super::maths::{Point, ln_3_f64};
use super::{Algorithm, PointData};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Original {}

impl Algorithm for Original {
    #[doc = r" Prepares the ``PointData`` to iterate"]
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        // The first iteration is easy
        point.value = point.origin;
        point.iter = 1;
        // Cardioid and period-2 bulb checks
        let c1 = point.origin.re - 0.25;
        let y2 = point.origin.im * point.origin.im;
        let q = c1 * c1 + y2;
        let p1 = point.origin.re + 1.0;
        if (q * (q + (c1)) <= 0.25 * y2) || (p1 * p1 + y2 <= 0.0625) {
            point.mark_infinite();
        }
    }

    #[doc = r" The iteration function"]
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        point.value = point.value * point.value + point.origin;
        point.iter += 1;
    }

    #[doc = r"Finalises the point data once a pixel has escaped"]
    #[inline]
    fn finish(&self, point: &mut PointData) {
        #![allow(clippy::cast_precision_loss)]
        #![allow(clippy::cast_possible_truncation)]
        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // A couple of extra iterations helps decrease the size of the error term
        self.iterate(point);
        self.iterate(point);
        // by the logarithm of a power law,
        // point.value.norm().ln().ln() === (point.value.norm_sqr().ln() * 0.5).ln())

        let result: f64 = f64::from(point.iter)
            - (point.value.norm_sqr().ln() * 0.5).ln() / std::f64::consts::LN_2;

        point.result = Some(result as f32);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: -1.0, im: 0.0 }
    }
    fn default_axes(&self) -> super::Point {
        Point { re: 4.0, im: 4.0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mandel3 {}

impl Algorithm for Mandel3 {
    // Default prepare

    #[doc = r" The iteration function"]
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // point.value = point.value * point.value * point.value + point.origin;
        // (it is 8% faster to unroll in this way)
        let (re, im) = (point.value.re, point.value.im);
        let (re2, im2) = (re * re, im * im);
        point.value.re = re * re2 - 3.0 * re * im2 + point.origin.re;
        point.value.im = 3.0 * im * re2 - im * im2 + point.origin.im;
        point.iter += 1;
    }

    #[doc = r"Finalises the point data once a pixel has escaped"]
    #[inline]
    fn finish(&self, point: &mut PointData) {
        #![allow(clippy::cast_precision_loss)]
        #![allow(clippy::cast_possible_truncation)]
        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // A couple of extra iterations helps decrease the size of the error term
        self.iterate(point);
        self.iterate(point);
        // logarithm of a power law applies here too

        // Even though result is f32, work in f64 here to avoid a numeric precision issue.
        // Otherwise, norm_sqr goes to inf much faster, which is unsightly with some colourers.
        let result: f64 =
            f64::from(point.iter) - (point.value.norm_sqr().ln() * 0.5).ln() / ln_3_f64();
        point.result = Some(result as f32);
    }
}
