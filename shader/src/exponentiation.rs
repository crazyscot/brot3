//! Exponentation strategies for fractals, as a trait to allow monomorphisation and inlining

use super::Complex;

pub trait Exponentiator: Copy + Clone {
    fn apply_to(self, z: Complex) -> Complex;
}
#[derive(Copy, Clone, Debug)]
pub struct Exp2;
#[derive(Copy, Clone, Debug)]
pub struct ExpIntN(pub i32);
#[derive(Copy, Clone, Debug)]
pub struct ExpFloat(pub f32);

// special case for exponent 2, which is the most common and can be optimised to a simple complex multiplication
impl Exponentiator for Exp2 {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z * z
    }
}

impl Exponentiator for ExpIntN {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z.powi(self.0).to_rectangular()
    }
}

impl Exponentiator for ExpFloat {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z.powf(self.0).to_rectangular()
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Complex, Exp2, ExpFloat, ExpIntN, Exponentiator};
    use assert_float_eq::assert_float_absolute_eq;
    use pretty_assertions::assert_eq;

    macro_rules! assert_complex_ulps_eq {
        ($a:expr, $b:expr) => {
            assert_float_absolute_eq!($a.re, $b.re);
            assert_float_absolute_eq!($a.im, $b.im);
        };
    }

    #[test]
    fn two() {
        let e2 = Exp2;
        let ei = ExpIntN(2);
        let ef = ExpFloat(2.0);

        // known answers, a basic sanity check that Complex and Exponentiator work
        let input = Complex::new(10., 0.);
        let expected = Complex::new(100., 0.);
        assert_eq!(e2.apply_to(input), expected);
        assert_eq!(ei.apply_to(input), expected);
        assert_eq!(ef.apply_to(input), expected);

        let input = Complex::new(2., 2.);
        let expected = Complex::new(0., 8.);
        assert_eq!(e2.apply_to(input), expected);
        assert_complex_ulps_eq!(ei.apply_to(input), expected);
        assert_complex_ulps_eq!(ef.apply_to(input), expected);

        let input = Complex::new(0., 1.);
        let expected = Complex::new(-1., 0.);
        assert_eq!(e2.apply_to(input), expected);
        assert_complex_ulps_eq!(ei.apply_to(input), expected);
        assert_complex_ulps_eq!(ef.apply_to(input), expected);
    }
}
