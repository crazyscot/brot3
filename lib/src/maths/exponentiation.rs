//! Exponentiation strategies for fractals, as a trait to allow monomorphisation and inlining
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

use num_traits::AsPrimitive as _;

use crate::Complex;
#[cfg(spirv)]
use crate::Real;

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
 * We used to have separate IntegerPower, RealPower and ComplexPower structs.
 * Integer benchmarked much slower than Real/ComplexPower on both CPU and GPU.
 * This appears to be because `powi` becomes a software loop, whereas RealPower uses hardware
 * powf instructions. RealPower benchmarked about the same as Complex, but there was no point in
 * keeping it separate.
 */

#[derive(Copy, Clone, Debug)]
pub struct RealPower(pub f32);
impl Exponentiator for RealPower {
    #[inline]
    fn apply_to(self, z: Complex) -> Complex {
        z.powf(self.0).to_rectangular()
    }

    #[allow(clippy::float_cmp)]
    #[inline]
    fn apply_power_minus_1_to(self, z: Complex) -> Complex {
        z.powf(self.0 - 1.0).to_rectangular()
    }

    fn power(self) -> f32 {
        self.0
    }

    fn log2(self) -> f32 {
        self.0.log2()
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
        maths::{Exponentiator, Power2, Power3, Power4, RealPower},
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
        let rp = RealPower(2.5);
        let z = Complex::new(1.0, 1.0);
        let z1 = rp.apply_to(z);
        assert_float_eq!(z1.re, -0.91018, abs <= 0.000_1);
        assert_float_eq!(z1.im, 2.19737, abs <= 0.000_1);
        assert_eq!(rp.power(), 2.5);
        assert_float_eq!(rp.log2(), 1.32192, abs <= 0.000_1);
    }

    #[test]
    fn complex_basics() {
        let ec = RealPower(2.0);
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
    fn apply_power_minus_1_to_power2() {
        let exp = Power2 {};
        let z = Complex::new(2.0, 0.0);
        let z_pow_minus_1 = exp.apply_power_minus_1_to(z);
        // 2^(2-1) = 2^1 = 2
        assert_complex_eq!(z_pow_minus_1, Complex::new(2.0, 0.0));
    }

    #[test]
    fn apply_power_minus_1_to_power3() {
        let exp = Power3 {};
        let z = Complex::new(2.0, 0.0);
        let z_pow_minus_1 = exp.apply_power_minus_1_to(z);
        // 2^(3-1) = 2^2 = 4
        assert_complex_eq!(z_pow_minus_1, Complex::new(4.0, 0.0));
    }

    #[test]
    fn apply_power_minus_1_to_real_power() {
        let exp = RealPower(3.5);
        let z = Complex::new(2.0, 0.0);
        let z_pow_minus_1 = exp.apply_power_minus_1_to(z);
        // 2^(3.5-1) = 2^2.5 ≈ 5.657
        let z_pow_2_5 = RealPower(2.5).apply_to(z);
        assert_complex_eq!(z_pow_minus_1, z_pow_2_5);
    }

    #[test]
    fn apply_power_minus_1_to_integer_powers() {
        let z = Complex::new(1.5, 0.0);

        let p2 = Power2 {};
        let p2_result = p2.apply_power_minus_1_to(z);
        assert_complex_eq!(p2_result, z); // 1.5^1

        let p3 = Power3 {};
        let p3_result = p3.apply_power_minus_1_to(z);
        let expected = z * z; // 1.5^2
        assert_complex_eq!(p3_result, expected);

        let p4 = Power4 {};
        let p4_result = p4.apply_power_minus_1_to(z);
        let expected = z * z * z; // 1.5^3
        assert_complex_eq!(p4_result, expected);
    }
}
