//! Fractal algorithms.
//! Can also be called on the host.

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Complex, FragmentConstants, PointResult, Vec2};
use crate::exponentiation::Exponentiator;

use core::marker::PhantomData;

pub fn render(constants: &FragmentConstants, point: Vec2) -> PointResult {
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

        let exp_ln = match self.constants.exponent.typ {
            NumericType::Integer if self.constants.exponent.int == 2 => core::f32::consts::LN_2,
            NumericType::Integer => {
                let i = self.constants.exponent.int as f32;
                i.abs().ln()
            }
            NumericType::Float => {
                let f = self.constants.exponent.real;
                f.abs().ln()
            }
            NumericType::Complex => {
                let c = crate::exponentiation::ExpComplex::from(self.constants.exponent);
                // for now, we'll take abs(z) so we can take a log in |R.
                // c.0.abs().ln() === (c.0.abs_sq() ^ 0.5).ln() === 0.5 * c.0.abs_sq().ln()
                0.5 * c.0.abs_sq().ln()
            }
        };

        // by the logarithm of a power law,
        // z.norm().ln().ln() === (z.norm_sqr().ln() * 0.5).ln()
        let log_zn = z.abs_sq().ln() * 0.5;
        let nu = (log_zn / exp_ln).ln() / exp_ln;
        let smoothed_iters = (iters) as f32 + 2. - nu;

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

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{fractal, vec2, FragmentConstants, Vec2};
    use shader_common::{Algorithm, NumericType, Palette, PushExponent};
    use shader_util::Size;

    use pretty_assertions::assert_eq;

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            viewport_translate: vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(1, 1),
            max_iter: 10,
            needs_reiterate: true.into(),
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
            palette: Palette::default(),
            fractional_iters: true.into(),
            inspector_active: false.into(),
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn mandelbrot_known_answer() {
        let point = crate::vec2(-0.75, 0.75);
        eprintln!("{:#?}", test_frag_consts());
        let result = fractal::render(&test_frag_consts(), point);
        eprintln!("{result:?}");
        // expected result created by previous brot3 engine (adapted to this incarnation's maths)
        assert_eq!(result.fractional_iters, 5.6856737);
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
        // expected result created by previous brot3 engine (adapted to this incarnation's maths)
        assert_eq!(result.fractional_iters, 5.685674);
    }
}
