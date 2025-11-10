//! Exponentation strategies for fractals, as a trait to allow monomorphisation and inlining

use super::Complex;
use shader_common::PushExponent;

pub trait Exponentiator: Copy + Clone {
    fn apply_to(self, z: Complex) -> Complex;
    /// For the function z := z^k + c, what is the derivative of the z^k term?
    fn derivative(self) -> Complex;
}
/// Special case for raising to the power 2
#[derive(Copy, Clone, Debug)]
pub struct Exp2;
/// Integer power (`powi` function)
#[derive(Copy, Clone, Debug)]
pub struct ExpIntN(pub i32);

/// Real number power (`powf` function)
#[derive(Copy, Clone, Debug)]
pub struct ExpFloat(pub f32);
/// Complex power (`powc` function)
///
/// <div class="warning">
/// The complex logarithm is multi-valued. We use the principal value only in computing the power.
/// </div>
#[derive(Copy, Clone, Debug)]
pub struct ExpComplex(pub Complex);

// special case for exponent 2, which is the most common and can be optimised to a simple complex multiplication
impl Exponentiator for Exp2 {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z * z
    }
    #[inline(always)]
    fn derivative(self) -> Complex {
        Complex { re: 2.0, im: 0.0 }
    }
}

impl Exponentiator for ExpIntN {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z.powi(self.0).to_rectangular()
    }
    #[inline(always)]
    fn derivative(self) -> Complex {
        Complex {
            re: self.0 as f32 - 1.0,
            im: 0.0,
        }
    }
}

impl Exponentiator for ExpFloat {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        // Special case as this seems to cause a shader abort on the GPU
        // when z==0.0... This isn't surprising as 0^0 is strictly undefined.
        if self.0 == 0.0 {
            return Complex::ONE;
        }
        z.powf(self.0).to_rectangular()
    }
    #[inline(always)]
    fn derivative(self) -> Complex {
        Complex {
            re: self.0 - 1.0,
            im: 0.0,
        }
    }
}

impl Exponentiator for ExpComplex {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        // special case as ln(0) is undefined
        if z == Complex::ZERO {
            return Complex::ZERO;
        }
        // special case to avoid breaking at 0^0 (undefined)
        if self.0 == Complex::ZERO {
            return Complex::ONE;
        }
        // function: z^p = e^(p ln(z))
        (self.0 * z.ln()).exp().to_rectangular()
    }
    #[inline(always)]
    fn derivative(self) -> Complex {
        Complex {
            re: self.0.re - 1.0,
            im: self.0.im,
        }
    }
}

impl From<PushExponent> for ExpComplex {
    fn from(value: PushExponent) -> Self {
        Self(Complex {
            re: value.real,
            im: value.imag,
        })
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Complex, Exp2, ExpComplex, ExpFloat, ExpIntN, Exponentiator};
    use float_eq::{assert_float_eq, float_ne};
    use pretty_assertions::assert_eq;

    macro_rules! assert_complex_eq {
        ($a:expr, $b:expr) => {
            assert_float_eq!($a.re, $b.re, abs <= 0.000_001);
            assert_float_eq!($a.im, $b.im, abs <= 0.000_001);
        };
    }
    macro_rules! assert_complex_ne {
        ($a:expr, $b:expr) => {
            assert!(
                float_ne!($a.re, $b.re, abs <= 0.000_001)
                    || float_ne!($a.im, $b.im, abs <= 0.000_001),
                "complexes are too close: {} vs {}",
                $a,
                $b
            );
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
        assert_complex_eq!(ei.apply_to(input), expected);
        assert_complex_eq!(ef.apply_to(input), expected);

        let input = Complex::new(0., 1.);
        let expected = Complex::new(-1., 0.);
        assert_eq!(e2.apply_to(input), expected);
        assert_complex_eq!(ei.apply_to(input), expected);
        assert_complex_eq!(ef.apply_to(input), expected);
    }

    #[test]
    fn complex_basics() {
        let e2 = Exp2;
        let ec = ExpComplex(Complex::from(2.0));

        let mut z = Complex::new(0., 1.);
        // Test that i^2 = -1 and that -1^2 = 1:
        for _ in 0..2 {
            let z2 = e2.apply_to(z);
            let zc = ec.apply_to(z);
            assert_complex_eq!(z2, zc);
            assert_complex_ne!(z, z2);
            z = z2;
        }

        let z = crate::vec2(-0.75, 0.75).into();
        let z2 = e2.apply_to(z);
        let zc = ec.apply_to(z);
        println!("{zc}");
        assert_complex_eq!(z2, zc);
    }
    #[test]
    fn powc_known_answer() {
        let z = Complex::new(2.0, 3.0);
        let exp = ExpComplex(Complex::new(0.5, -0.707));
        let expected = Complex::new(3.4806898, -1.5348526);
        let result = exp.apply_to(z);
        assert_complex_eq!(result, expected);
        println!("{z} ^ {} = {result}", exp.0);
    }
}
