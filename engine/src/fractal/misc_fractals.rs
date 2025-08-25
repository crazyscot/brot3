// Miscellaneous fractals
// (c) 2024 Ross Younger

use super::{IAlgorithm, Point, PointData, mandelbrot::Original};

/// Prep function for fractals which appear upside down in this coordinate system
/// (i.e. invert them)
#[inline]
fn prepare_upside_down(point: &mut PointData) {
    let origin = point.origin.conj();
    point.origin = origin;
    point.value = origin;
    point.iter = 1;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mandelbar {
    delegate: Original,
}

impl IAlgorithm for Mandelbar {
    // Standard prepare

    #[inline]
    fn iterate(&self, point: &mut PointData) {
        let conjugate = point.value.conj();
        point.value = conjugate * conjugate + point.origin;
        point.iter += 1;
    }

    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    // standard centre 0+0i

    fn default_axes(&self) -> Point {
        Point { re: 5.0, im: 5.0 }
    }
}

// ///////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BurningShip {
    delegate: Original,
}

impl IAlgorithm for BurningShip {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        prepare_upside_down(point);
    }

    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // z:=(|Re(z)|+i|Im(z)|)^2+c
        let modpt = Point {
            re: point.value.re.abs(),
            im: point.value.im.abs(),
        };
        point.value = modpt * modpt + point.origin;
        point.iter += 1;
    }

    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: -0.5, im: 0.5 }
    }

    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

// ///////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Celtic {
    delegate: Original,
}

impl IAlgorithm for Celtic {
    // standard prepare

    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // z:= (|Re(z^2)| + i.Im(z^2) + c)
        let z2 = point.value * point.value;
        point.value = Point {
            re: z2.re.abs() + point.origin.re,
            im: z2.im + point.origin.im,
        };
        point.iter += 1;
    }

    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: -1.0, im: 0.0 }
    }

    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

// ///////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variant {
    delegate: Original,
}

impl IAlgorithm for Variant {
    // standard prepare

    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // z:=z^2+c with Re(z):=|Re(z)| on odd iterations
        let z2 = point.value * point.value;
        point.value = if (point.iter % 2) == 1 {
            Point {
                re: z2.re.abs(),
                im: z2.im,
            } + point.origin
        } else {
            z2 + point.origin
        };
        point.iter += 1;
    }

    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    fn default_centre(&self) -> super::Point {
        Point { re: -1.0, im: 0.0 }
    }

    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

// ///////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BirdOfPrey {
    delegate: Original,
}

impl IAlgorithm for BirdOfPrey {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        prepare_upside_down(point);
    }
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // z:=(Re(z)+i|Im(z)|)^2+c"
        let modpt = Point {
            re: point.value.re,
            im: point.value.im.abs(),
        };
        point.value = modpt * modpt + point.origin;
        point.iter += 1;
    }

    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    // standard centre 0+0i

    fn default_axes(&self) -> Point {
        Point { re: 5.0, im: 5.0 }
    }
}

// ///////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Buffalo {
    delegate: Original,
}

impl IAlgorithm for Buffalo {
    #[inline]
    fn prepare(&self, point: &mut PointData) {
        prepare_upside_down(point);
    }
    #[inline]
    fn iterate(&self, point: &mut PointData) {
        // z:=|z|^2 - |z| + c
        let z = Point {
            re: point.value.re.abs(),
            im: point.value.im.abs(),
        };
        point.value = z * z - z + point.origin;
        point.iter += 1;
    }
    #[inline]
    fn finish(&self, point: &mut PointData) {
        self.delegate.finish(point);
    }

    // standard centre 0+0i

    fn default_axes(&self) -> Point {
        Point { re: 4.0, im: 4.0 }
    }
}

// ///////////////////////////////////////////////////////////
