#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Builder, Complex, FractalResult, FragmentConstants, RenderData, Vec2};

use crate::exponentiation::Exponentiator;

pub(super) fn render(constants: &FragmentConstants, point: Vec2) -> RenderData {
    use shader_common::{Algorithm, NumericType};
    let c = Complex::from(point);
    macro_rules! builder {
        ($fractal:ident, $c_value:expr) => {{
            match constants.exponent.typ {
                NumericType::Integer if constants.exponent.int == 2 => Builder {
                    constants,
                    fractal: $fractal {
                        c: $c_value,
                        exponent: crate::exponentiation::Exp2,
                    },
                    _phantom: core::marker::PhantomData,
                }
                .iterations(),
                NumericType::Integer => Builder {
                    constants,
                    fractal: $fractal {
                        c: $c_value,
                        exponent: crate::exponentiation::ExpIntN(constants.exponent.int),
                    },
                    _phantom: core::marker::PhantomData,
                }
                .iterations(),
                NumericType::Float => Builder {
                    constants,
                    fractal: $fractal {
                        c: $c_value,
                        exponent: crate::exponentiation::ExpFloat(constants.exponent.float),
                    },
                    _phantom: core::marker::PhantomData,
                }
                .iterations(),
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

pub(crate) trait Fractal<E: Exponentiator>: private::Modifier<E> {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult;
    fn iterate_inner(&self, c: Complex, constants: &FragmentConstants, exp: E) -> FractalResult {
        use shader_common::NumericType;

        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let max_iter = constants.max_iter;
        // TODO: Cardoid and period-2 bulb checks in original?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            self.modify(&mut z);
            z = self.iterate_algorithm(exp, z, c, iters);
            iters += 1;
            norm_sqr = z.abs_sq();
        }
        let inside = iters == max_iter && (norm_sqr < ESCAPE_THRESHOLD_SQ);

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html

        let (exponent, ln_exponent) = match constants.exponent.typ {
            NumericType::Integer if constants.exponent.int == 2 => (2., core::f32::consts::LN_2),
            NumericType::Integer => (
                constants.exponent.int as f32,
                (constants.exponent.int as f32).abs().ln(),
            ),
            NumericType::Float => (
                constants.exponent.float,
                constants.exponent.float.abs().ln(),
            ),
        };

        if exponent.abs() < 4. {
            // A couple of extra iterations helps decrease the size of the error term
            // This does not work above about exp=3, as the repeated iterations quickly cause overflow
            z = self.iterate_algorithm(exp, z, c, iters);
            iters += 1;
            z = self.iterate_algorithm(exp, z, c, iters);
            iters += 1;
        }
        // by the logarithm of a power law,
        // z.norm().ln().ln() === (z.norm_sqr().ln() * 0.5).ln())
        let smoothed_iters = (iters) as f32 - (z.abs_sq().ln() * 0.5).ln() / ln_exponent;

        FractalResult {
            inside,
            iters,
            smoothed_iters,
        }
    }
}

mod private {
    use crate::exponentiation::Exponentiator;

    use super::Complex;
    pub trait Modifier<E: Exponentiator> {
        /// Override as necessary
        #[inline(always)]
        fn modify(&self, _z: &mut Complex) {}

        // Override as necessary
        #[inline(always)]
        fn iterate_algorithm(&self, e: E, z: Complex, c: Complex, _iters: u32) -> Complex {
            e.apply_to(z) + c
        }
    }
}

macro_rules! standard_fractal {
    ($name: ident) => {
        struct $name<E: Exponentiator> {
            pub c: Complex,
            pub exponent: E,
        }
        impl<E: Exponentiator> Fractal<E> for $name<E> {
            fn iterate(&self, constants: &FragmentConstants) -> FractalResult {
                self.iterate_inner(self.c, constants, self.exponent)
            }
        }
    };
}

standard_fractal!(Mandelbrot);
impl<E: Exponentiator> private::Modifier<E> for Mandelbrot<E> {}

standard_fractal!(Mandelbar);
impl<E: Exponentiator> private::Modifier<E> for Mandelbar<E> {
    // Same as mandelbrot, but conjugate c each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        *z = z.conjugate();
    }
}

standard_fractal!(BurningShip);
impl<E: Exponentiator> private::Modifier<E> for BurningShip<E> {
    // Same as mandelbrot, but take abs(re) and abs(im) each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        z.re = z.re.abs();
        z.im = z.im.abs();
    }
}

standard_fractal!(Celtic);
impl<E: Exponentiator> private::Modifier<E> for Celtic<E> {
    #[inline(always)]
    fn iterate_algorithm(&self, e: E, z: Complex, c: Complex, _iters: u32) -> Complex {
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

standard_fractal!(BirdOfPrey);
impl<E: Exponentiator> private::Modifier<E> for BirdOfPrey<E> {
    // Same as mandelbrot, but take abs(im) each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        z.im = z.im.abs();
    }
}

standard_fractal!(Variant);
impl<E: Exponentiator> private::Modifier<E> for Variant<E> {
    #[inline(always)]
    fn iterate_algorithm(&self, e: E, z: Complex, c: Complex, iters: u32) -> Complex {
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
