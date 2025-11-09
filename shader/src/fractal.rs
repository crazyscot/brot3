//! Fractal algorithms.
//! Can also be called on the host.

#[cfg(not(target_arch = "spirv"))]
const DEBUG_FRACTAL: bool = false;

macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "spirv"))]
        if DEBUG_FRACTAL {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Complex, FragmentConstants, PointResult, Vec2};
use crate::exponentiation::Exponentiator;

use core::marker::PhantomData;

pub fn render(constants: &FragmentConstants, point: Vec2) -> PointResult {
    use shader_common::{enums::Algorithm, NumericType};
    let c = Complex::from(point);
    macro_rules! builder {
        ($fractal:ident, $c_value:expr) => {{
            match constants.exponent.typ {
                NumericType::Integer if constants.exponent.int == 2 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    expo: crate::exponentiation::Exp2,
                    c: $c_value,
                }
                .run(),
                NumericType::Integer => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    expo: crate::exponentiation::ExpIntN(constants.exponent.int),
                    c: $c_value,
                }
                .run(),
                NumericType::Float => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    expo: crate::exponentiation::ExpFloat(constants.exponent.real),
                    c: $c_value,
                }
                .run(),
                NumericType::Complex => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    expo: crate::exponentiation::ExpComplex::from(constants.exponent),
                    c: $c_value,
                }
                .run(),
                _ => todo!(),
            }
        }};
    }
    match constants.algorithm {
        Algorithm::Mandelbrot => builder!(Mandelbrot, c),
        // Mandeldrop is the same algorithm but with a different c
        Algorithm::Mandeldrop => builder!(Mandelbrot, c.recip()),
        Algorithm::Mandelbar => builder!(Mandelbar, c),
        Algorithm::BurningShip => builder!(BurningShip, c),
        Algorithm::Celtic => builder!(Celtic, c),
        Algorithm::Variant => builder!(Variant, c),
        Algorithm::BirdOfPrey => builder!(BirdOfPrey, c),
        _ => todo!(),
    }
}

struct Runner<'a, F, E>
where
    F: AlgorithmDetail<E>,
    E: Exponentiator,
{
    constants: &'a FragmentConstants,
    algo: PhantomData<F>,
    expo: E,
    c: Complex,
}

impl<F, E> Runner<'_, F, E>
where
    F: AlgorithmDetail<E>,
    E: Exponentiator,
{
    fn run(self) -> PointResult {
        use shader_common::NumericType;

        const ESCAPE_THRESHOLD: f32 = 10.0;
        const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
        let loglog2_escape_threshold: f32 = ESCAPE_THRESHOLD.log2().log2();

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut dz = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let max_iter = self.constants.max_iter;

        deprintln!("DBG: run for c={:?}", self.c);
        // TODO: Cardoid and period-2 bulb checks in original?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            F::pre_modify_point(&mut z);
            (z, dz) = F::iterate_algorithm(z, dz, self.expo, self.c, iters);
            iters += 1;
            norm_sqr = z.abs_sq();
            deprintln!("DBG: iters={iters}, z={z}, dz={dz}, |z|^2={norm_sqr}");
        }
        let inside = iters == max_iter && (norm_sqr < ESCAPE_THRESHOLD_SQ);

        // distance estimate, angle
        let za = z.abs();
        let distance = (za * za).ln() * za / dz.abs();
        let angle = z.arg();

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // The log(exponent) term is necessary for powers other than 2
        let log2_exponent = match self.constants.exponent.typ {
            NumericType::Integer if self.constants.exponent.int == 2 => 1.0,
            NumericType::Integer => {
                let i = self.constants.exponent.int as f32;
                i.abs().log2()
            }
            NumericType::Float => {
                let f = self.constants.exponent.real;
                f.abs().log2()
            }
            NumericType::Complex => {
                let c = crate::exponentiation::ExpComplex::from(self.constants.exponent);
                // for now, we'll take abs(z) so we can take a log in |R.
                // c.0.abs().log() === (c.0.abs_sq() ^ 0.5).log() === 0.5 * c.0.abs_sq().log()
                0.5 * c.0.abs_sq().log2()
            }
            _ => todo!(),
        };
        // by the logarithm of a power law,
        // z.norm().log() === z.norm_sqr().log() * 0.5
        let log_zn = z.abs_sq().log2() * 0.5;
        let smoothed_iters = 1. + loglog2_escape_threshold - log_zn.log2() / log2_exponent;

        if inside {
            PointResult::new_inside(distance, angle, norm_sqr)
        } else {
            PointResult::new_outside(iters, smoothed_iters, distance, angle, norm_sqr)
        }
    }
}

pub(crate) trait AlgorithmDetail<E: Exponentiator> {
    /// Pre-modifies a point before applying the algorithm.
    ///
    /// Override as necessary.
    #[inline(always)]
    fn pre_modify_point(_z: &mut Complex) {}

    /// One iteration of the fractal algorithm.
    ///
    /// The provided implementation computes `z := z.pow(e) + c`, but this doesn't
    /// suit all algorithms. Override as necessary.
    #[inline(always)]
    fn iterate_algorithm(
        z: Complex,
        dz: Complex,
        e: E,
        c: Complex,
        _iters: u32,
    ) -> (Complex /*z*/, Complex /*dz*/) {
        let dz = 2.0 * z * dz + 1.0; // TODO: Does this change when exp != 2?
        let z = e.apply_to(z) + c;
        (z, dz)
    }
}

struct Mandelbrot {}
impl<E: Exponentiator> AlgorithmDetail<E> for Mandelbrot {}

struct Mandelbar {}
impl<E: Exponentiator> AlgorithmDetail<E> for Mandelbar {
    // Same as mandelbrot, but conjugate c each time
    #[inline(always)]
    fn pre_modify_point(z: &mut super::Complex) {
        *z = z.conjugate();
    }
}

struct BurningShip {}
impl<E: Exponentiator> AlgorithmDetail<E> for BurningShip {
    // Same as mandelbrot, but take abs(re) and abs(im) each time
    #[inline(always)]
    fn pre_modify_point(z: &mut super::Complex) {
        z.re = z.re.abs();
        z.im = z.im.abs();
    }
}

struct Celtic {}
impl<E: Exponentiator> AlgorithmDetail<E> for Celtic {
    #[inline(always)]
    fn iterate_algorithm(
        z: Complex,
        dz: Complex,
        e: E,
        c: Complex,
        _iters: u32,
    ) -> (Complex, Complex) {
        // Based on mandelbrot, but using the formula:
        //   z := abs(re(z^2)) + i.im(z^2) + c
        let dz = 2.0 * z * dz + 1.0; // TODO: correct for non-2 exponents?
        let zz = e.apply_to(z);
        let z2 = Complex {
            re: zz.re.abs(),
            im: zz.im,
        };
        (z2 + c, dz)
    }
}

struct BirdOfPrey {}
impl<E: Exponentiator> AlgorithmDetail<E> for BirdOfPrey {
    // Same as mandelbrot, but take abs(im) each time
    #[inline(always)]
    fn pre_modify_point(z: &mut super::Complex) {
        z.im = z.im.abs();
    }
}

struct Variant {}
impl<E: Exponentiator> AlgorithmDetail<E> for Variant {
    #[inline(always)]
    fn iterate_algorithm(
        z: Complex,
        dz: Complex,
        e: E,
        c: Complex,
        iters: u32,
    ) -> (Complex, Complex) {
        let dz = 2.0 * z * dz + 1.0; // TODO: correct for non-2 exponents?
        let zz = e.apply_to(z);
        let z = if (iters % 2) == 1 {
            Complex {
                re: zz.re.abs() + c.re,
                im: zz.im + c.im,
            }
        } else {
            zz + c
        };
        (z, dz)
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{fractal, vec2, FragmentConstants, Vec2};
    use const_default::ConstDefault as _;
    use shader_common::{enums::Algorithm, Flags, NumericType, Palette, PushExponent};
    use shader_util::Size;

    use pretty_assertions::assert_eq;

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            flags: Flags::NEEDS_REITERATE,
            viewport_translate: vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(1, 1),
            max_iter: 10,
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
            palette: Palette::DEFAULT,
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn mandelbrot_known_answer() {
        let point = crate::vec2(-0.75, 0.75);
        eprintln!("{:#?}", test_frag_consts());
        let result = fractal::render(&test_frag_consts(), point);
        eprintln!("{result:?}");
        assert_eq!(result.iters_fraction(), 0.52201414);
    }
    #[test]
    fn mandelbrot_known_answer_cpow() {
        let point = crate::vec2(-0.75, 0.75);
        let mut consts = test_frag_consts();
        consts.exponent.typ = NumericType::Complex;
        consts.exponent.real = 2.0;
        consts.exponent.imag = 0.0;
        eprintln!("{consts:#?}");
        let result = fractal::render(&consts, point);
        eprintln!("{result:?}");
        assert_eq!(result.iters_fraction(), 0.5220146);
    }
}
