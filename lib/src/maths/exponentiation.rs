//! Exponentiation strategies for fractals, as a trait to allow monomorphisation and inlining
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

#[cfg(spirv)]
use crate::Real;
use crate::{Complex, data::PushExponent};

pub trait Exponentiator: Copy + Clone {
    fn apply_to(self, z: Complex) -> Complex;
    /// For the function z := z^k + c, what is the real power k so that we can compute the
    /// derivative?
    fn power(self) -> f32;
    /// What is the log2 of the exponent?
    fn log2(self) -> f32;
}

macro_rules! power_unrolled {
    ($pow:literal, $unroll:expr) => {
        paste::paste! {
            #[derive(Copy, Clone, Debug)]
            pub struct [<Power $pow>] {}
            impl Exponentiator for [<Power $pow>] {
                fn apply_to(self, z: Complex) -> Complex {
                    $unroll(z)
                }
                #[allow(clippy::cast_precision_loss)]
                fn power(self) -> f32 {
                    $pow as f32
                }
                #[allow(clippy::cast_precision_loss)]
                fn log2(self) -> f32 {
                    ($pow as f32).log2()
                }
            }
        }
    };
}

macro_rules! int_powers {
    ($(($pow:literal, $unroll:expr)),+) => {
        $(power_unrolled!($pow, $unroll);)+
    }
}

int_powers!(
    (2, |z| z * z),
    (3, |z| z * z * z),
    (4, |z| z * z * z * z),
    (5, |z| z * z * z * z * z),
    (6, |z| z * z * z * z * z * z)
);

#[derive(Copy, Clone, Debug)]
pub struct IntegerPower(pub i32);
impl Exponentiator for IntegerPower {
    fn apply_to(self, z: Complex) -> Complex {
        match self.0 {
            0 => {
                if z == Complex::ZERO {
                    Complex::ZERO
                } else {
                    Complex::ONE
                }
            }
            _ => z.powi(self.0).to_rectangular(),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn power(self) -> f32 {
        self.0 as f32
    }

    #[allow(clippy::cast_precision_loss)]
    fn log2(self) -> f32 {
        // special case where exponent is less than 2, to avoid undefinedness at/below 0.
        (self.0 as f32).max(2.0).log2()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RealPower(pub f32);
impl Exponentiator for RealPower {
    fn apply_to(self, z: Complex) -> Complex {
        if self.0 == 0.0 && z == Complex::ZERO {
            Complex::ZERO
        } else {
            z.powf(self.0).to_rectangular()
        }
    }

    fn power(self) -> f32 {
        self.0
    }

    fn log2(self) -> f32 {
        // special case where exponent is less than 2, to avoid undefinedness at/below 0.
        self.0.max(2.0).log2()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComplexPower(pub Complex);
impl Exponentiator for ComplexPower {
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

    use float_eq::{assert_float_eq, float_ne};
    use pretty_assertions::assert_eq;

    use crate::{
        Complex,
        data::PushExponent,
        maths::{ComplexPower, Exponentiator, IntegerPower, Power2, RealPower},
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
    #[allow(clippy::float_cmp, clippy::cast_precision_loss)]
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
            assert_eq!(pow, i as f32);
            assert_eq!(log2, (i as f32).log2());
            assert_eq!(z.im, 0.0);
            assert_eq!(z.re, 2.0_f32.powi(i));
        }
    }

    #[test]
    #[allow(clippy::float_cmp, clippy::cast_precision_loss)]
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
        let e2 = IntegerPower(2);
        let ec = ComplexPower(Complex::from(2.0));
        let sc = Power2 {};

        let mut z = Complex::new(0., 1.);
        // Test that i^2 = -1 and that -1^2 = 1:
        for _ in 0..2 {
            let z2 = e2.apply_to(z);
            let zc = ec.apply_to(z);
            let zsc = sc.apply_to(z);
            assert_complex_eq!(z2, zc);
            assert_complex_eq!(z2, zsc);
            assert_complex_ne!(z, z2);
            z = z2;
        }

        let z = crate::vec2(-0.75, 0.75).into();
        let z2 = e2.apply_to(z);
        let zc = ec.apply_to(z);
        let zsc = sc.apply_to(z);
        println!("{zc}");
        assert_complex_eq!(z2, zc);
        assert_complex_eq!(z2, zsc);
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
        let expf = RealPower(0.0);
        let two = Complex::ONE * 2.0;

        // x^0 == 0
        let f1 = expf.apply_to(two);
        assert_eq!(f1, Complex::ONE);
        // 0^0 is undefined, but in our world we've special-cased it as zero to prevent a shader
        // abort.
        let z2 = expf.apply_to(Complex::ZERO);
        assert_eq!(z2, Complex::ZERO);

        // Consistency check with integer powers
        let exp_int = IntegerPower(0);
        let i1 = exp_int.apply_to(two);
        assert_eq!(i1, Complex::ONE);
        let i2 = exp_int.apply_to(Complex::ZERO);
        assert_eq!(i2, Complex::ZERO);
        let i3 = Power2 {}.apply_to(Complex::ZERO);
        assert_eq!(i3, Complex::ZERO);

        // Now do it all again with complex powers
        let exp_complex = ComplexPower(Complex::ZERO);
        let z1 = exp_complex.apply_to(two);
        assert_eq!(z1, Complex::ONE);
        let z2 = exp_complex.apply_to(Complex::ZERO);
        assert_eq!(z2, Complex::ZERO);
    }
}
