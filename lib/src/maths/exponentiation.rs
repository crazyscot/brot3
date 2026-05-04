//! Exponentiation strategies for fractals, as a trait to allow monomorphisation and inlining
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

use num_traits::AsPrimitive as _;

#[cfg(spirv)]
use crate::Real;
use crate::{Complex, data::PushExponent};

pub trait Exponentiator: Copy + Clone {
    fn apply_to(self, z: Complex) -> Complex;
    fn apply_power_minus_1_to(self, z: Complex) -> Complex;
    /// For the function z := z^k + c, what is the real power k so that we can compute the
    /// derivative?
    fn power(self) -> f32;
    /// What is the log2 of the exponent?
    fn log2(self) -> f32;
}

macro_rules! unroll_int {
    ($pow:expr, $z:ident) => {
        match $pow {
            1 => $z,
            2 => $z * $z,
            3 => $z * $z * $z,
            4 => {
                let z2 = $z * $z;
                z2 * z2
            }
            5 => {
                let z2 = $z * $z;
                let z4 = z2 * z2;
                z4 * $z
            }
            6 => {
                let z2 = $z * $z;
                let z4 = z2 * z2;
                z4 * z2
            }
            _ => unreachable!(),
        }
    };
}

macro_rules! power_unrolled {
    ($pow:literal) => {
        paste::paste! {
            #[derive(Copy, Clone, Debug)]
            pub struct [<Power $pow>] {}
            impl Exponentiator for [<Power $pow>] {
                #[inline(always)]
                fn apply_to(self, z: Complex) -> Complex {
                    unroll_int!($pow, z)
                }
                #[inline(always)]
                fn apply_power_minus_1_to(self, z: Complex) -> Complex {
                    unroll_int!($pow - 1, z)
                }
                #[inline(always)]
                fn power(self) -> f32 {
                    // Cast with precision loss is OK here. Power is limited to 20.
                    $pow.as_()
                }
                #[inline(always)]
                fn log2(self) -> f32 {
                    self.power().log2()
                }
            }
        }
    };
}

macro_rules! int_powers {
    ($($pow:literal),+) => {
        $(power_unrolled!($pow);)+
    }
}

int_powers!(2, 3, 4, 5, 6);

/*
 * We used to have separate IntegerPower and RealPower structs.
 * Integer benchmarked much slower than ComplexPower on both CPU and GPU.
 * This appears to be because `powi` becomes a software loop, whereas RealPower uses hardware
 * powf instructions. RealPower benchmarked about the same, but there was no point in keeping it
 * separately.
 */

#[derive(Copy, Clone, Debug)]
pub struct ComplexPower(pub Complex);
impl ComplexPower {
    #[inline]
    fn apply(power: Complex, z: Complex) -> Complex {
        // special case as ln(0) is undefined
        if z == Complex::ZERO {
            return Complex::ZERO;
        }
        // special case to avoid breaking at 0^0 (undefined)
        if power == Complex::ZERO {
            return Complex::ONE;
        }
        // function: z^p = e^(p ln(z))
        (power * z.ln()).exp().to_rectangular()
    }
}
impl Exponentiator for ComplexPower {
    #[inline]
    fn apply_to(self, z: Complex) -> Complex {
        Self::apply(self.0, z)
    }

    #[inline]
    fn apply_power_minus_1_to(self, z: Complex) -> Complex {
        Self::apply(self.0 - Complex::ONE, z)
    }

    fn power(self) -> f32 {
        self.0.re
    }

    // For now, we'll compute a log in ℝ so take abs(power).
    // c.abs().log() === (c.abs_sq() ^ 0.5).log() === 0.5 * c.abs_sq().log()
    // For parity with Int and Floats, we'll special case where abs < 2 i.e. abs_sq < 4
    fn log2(self) -> f32 {
        self.0.abs_sq().max(4.0).log2() * 0.5
    }
}
impl From<PushExponent> for ComplexPower {
    fn from(exp: PushExponent) -> Self {
        Self(Complex {
            re: exp.real,
            im: exp.imag,
        })
    }
}
impl ComplexPower {
    #[must_use]
    pub fn new(real: f32, imag: f32) -> Self {
        Self(Complex { re: real, im: imag })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::cognitive_complexity)]

    use easy_cast::Conv as _;
    use float_eq::{assert_float_eq, float_ne};
    use pretty_assertions::assert_eq;

    use crate::{
        Complex,
        data::PushExponent,
        maths::{ComplexPower, Exponentiator, Power2},
    };

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
    #[allow(clippy::float_cmp)]
    fn int_basics() {
        let z_two = Complex::ONE + Complex::ONE;
        macro_rules! expo_object {
            ($expo:expr,$alg:ty) => {{
                let expo = $expo;
                (expo.apply_to(z_two), expo.power(), expo.log2())
            }};
        }
        for i in 2..=7 {
            let (z, pow, log2) = crate::exponent_monomorph!(
                PushExponent {
                    typ: crate::data::NumericType::Integer,
                    int: i,
                    ..Default::default()
                },
                expo_object,
                MandelbrotFamily
            );
            assert_eq!(pow, f32::conv(i));
            assert_eq!(log2, f32::conv(i).log2());
            assert_eq!(z.im, 0.0);
            assert_eq!(z.re, 2.0_f32.powi(i));
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn real_basics() {
        let rp = ComplexPower::new(2.5, 0.0);
        let z = Complex::new(1.0, 1.0);
        let z1 = rp.apply_to(z);
        assert_float_eq!(z1.re, -0.91018, abs <= 0.000_1);
        assert_float_eq!(z1.im, 2.19737, abs <= 0.000_1);
        assert_eq!(rp.power(), 2.5);
        assert_float_eq!(rp.log2(), 1.32192, abs <= 0.000_1);
    }

    #[test]
    fn complex_basics() {
        let ec = ComplexPower::new(2.0, 0.0);
        let sc = Power2 {};

        let mut z = Complex::new(0., 1.);
        // Test that i^2 = -1 and that -1^2 = 1:
        for _ in 0..2 {
            let zc = ec.apply_to(z);
            let zsc = sc.apply_to(z);
            assert_complex_eq!(zc, zsc);
            assert_complex_ne!(z, zc);
            z = zc;
        }

        let z = crate::vec2(-0.75, 0.75).into();
        let zc = ec.apply_to(z);
        let zsc = sc.apply_to(z);
        println!("{zc}");
        assert_complex_eq!(zc, zsc);
    }
    #[test]
    fn powc_known_answer() {
        let z = Complex::new(2.0, 3.0);
        let exp = ComplexPower(Complex::new(0.5, -0.707));
        let expected = Complex::new(3.480_689_8, -1.534_852_6);
        let result = exp.apply_to(z);
        assert_complex_eq!(result, expected);
        println!("{z} ^ {exp:?} = {result}");
    }

    #[test]
    fn power_zero_special_cases() {
        let two = Complex::ONE * 2.0;

        let exp_complex = ComplexPower(Complex::ZERO);
        // x^0 == 0
        let z1 = exp_complex.apply_to(two);
        assert_eq!(z1, Complex::ONE);
        // 0^0 is undefined, but in our world we've special-cased it as zero to prevent a shader
        // abort.
        let z2 = exp_complex.apply_to(Complex::ZERO);
        assert_eq!(z2, Complex::ZERO);
    }
}
