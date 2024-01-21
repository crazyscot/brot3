// Mandelbrot set implementation
// (c) 2024 Ross Younger

use super::{Algorithm, Point, PointData, Scalar, SCALAR_LN_2};

#[derive(Clone, Copy, Debug, Default)]
pub struct Original {}

impl Algorithm for Original {
    #[doc = r" Prepares the ``PointData`` to iterate"]
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
    fn finish(&self, point: &mut PointData) {
        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // A couple of extra iterations helps decrease the size of the error term
        self.iterate(point);
        self.iterate(point);
        point.result = Some(Scalar::from(point.iter) - point.value.norm().ln().ln() / SCALAR_LN_2);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: -1.0, im: 0.0 }
    }
    fn default_axes(&self) -> super::Point {
        Point { re: 4.0, im: 4.0 }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mandel3 {}

impl Algorithm for Mandel3 {
    // Default prepare

    #[doc = r" The iteration function"]
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        point.value = point.value * point.value * point.value + point.origin;
        point.iter += 1;
    }

    #[doc = r"Finalises the point data once a pixel has escaped"]
    fn finish(&self, point: &mut PointData) {
        const THREE: f64 = 3.0;
        let ln3: Scalar = THREE.ln(); // this should be a compile-time const

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // A couple of extra iterations helps decrease the size of the error term
        self.iterate(point);
        self.iterate(point);
        point.result = Some(Scalar::from(point.iter) - point.value.norm().ln().ln() / ln3);
    }
}
