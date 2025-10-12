//! Fractal algorithms.
//! Can also be called on the host.

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Complex, FragmentConstants, PointResult, Vec2};
use crate::exponentiation::Exponentiator;

use core::marker::PhantomData;

pub(super) fn render(constants: &FragmentConstants, point: Vec2) -> PointResult {
    use shader_common::{Algorithm, NumericType};
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
                    expo: crate::exponentiation::ExpFloat(constants.exponent.float),
                    c: $c_value,
                }
                .run(),
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

        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let max_iter = self.constants.max_iter;
        // TODO: Cardoid and period-2 bulb checks in original?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            F::pre_modify_point(&mut z);
            z = F::iterate_algorithm(self.expo, z, self.c, iters);
            iters += 1;
            norm_sqr = z.abs_sq();
        }
        let inside = iters == max_iter && (norm_sqr < ESCAPE_THRESHOLD_SQ);

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html

        let (exponent, ln_exponent) = match self.constants.exponent.typ {
            NumericType::Integer if self.constants.exponent.int == 2 => {
                (2., core::f32::consts::LN_2)
            }
            NumericType::Integer => (
                self.constants.exponent.int as f32,
                (self.constants.exponent.int as f32).abs().ln(),
            ),
            NumericType::Float => (
                self.constants.exponent.float,
                self.constants.exponent.float.abs().ln(),
            ),
        };

        if exponent.abs() < 4. {
            // A couple of extra iterations helps decrease the size of the error term
            // This does not work above about exp=3, as the repeated iterations quickly cause overflow
            z = F::iterate_algorithm(self.expo, z, self.c, iters);
            iters += 1;
            z = F::iterate_algorithm(self.expo, z, self.c, iters);
            iters += 1;
        }
        // by the logarithm of a power law,
        // z.norm().ln().ln() === (z.norm_sqr().ln() * 0.5).ln())
        let smoothed_iters = (iters) as f32 + 1. - (z.abs_sq().ln() * 0.5).ln() / ln_exponent;

        PointResult::new(inside, iters, smoothed_iters)
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
    fn iterate_algorithm(e: E, z: Complex, c: Complex, _iters: u32) -> Complex {
        e.apply_to(z) + c
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
    fn iterate_algorithm(e: E, z: Complex, c: Complex, _iters: u32) -> Complex {
        // Based on mandelbrot, but using the formula:
        //   z := abs(re(z^2)) + i.im(z^2) + c
        let zz = e.apply_to(z);
        let z2 = Complex {
            re: zz.re.abs(),
            im: zz.im,
        };
        z2 + c
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
    fn iterate_algorithm(e: E, z: Complex, c: Complex, iters: u32) -> Complex {
        let zz = e.apply_to(z);
        if (iters % 2) == 1 {
            Complex {
                re: zz.re.abs() + c.re,
                im: zz.im + c.im,
            }
        } else {
            zz + c
        }
    }
}
