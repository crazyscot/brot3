#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Builder, Complex, FractalResult, FragmentConstants, RenderData, Vec2};

pub(super) fn render(constants: &FragmentConstants, point: Vec2) -> RenderData {
    use shader_common::Algorithm;
    macro_rules! builder {
        ($fractal:expr) => {
            Builder {
                constants,
                fractal: $fractal,
            }
        };
    }
    let c = Complex::from(point);
    match constants.algorithm {
        Algorithm::Mandelbrot => builder!(Mandelbrot { c }).iterations(),
        // Mandeldrop is the same algorithm but with a different c
        Algorithm::Mandeldrop => builder!(Mandelbrot { c: c.recip() }).iterations(),
        Algorithm::Mandelbar => builder!(Mandelbar { c }).iterations(),
        Algorithm::BurningShip => builder!(BurningShip { c }).iterations(),
        Algorithm::Celtic => builder!(Celtic { c }).iterations(),
        Algorithm::Variant => builder!(Variant { c }).iterations(),
        Algorithm::BirdOfPrey => builder!(BirdOfPrey { c }).iterations(),
    }
}

pub(crate) trait Fractal: private::Modifier {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult;
    fn iterate_inner(&self, c: Complex, constants: &FragmentConstants) -> FractalResult {
        use shader_common::NumericType;

        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let max_iter = constants.max_iter;
        // TODO: Cardoid and period-2 bulb checks in original?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            self.modify(&mut z);
            z = self.iterate_algorithm(constants, z, c, iters);
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
            z = self.iterate_algorithm(constants, z, c, iters);
            iters += 1;
            z = self.iterate_algorithm(constants, z, c, iters);
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
    use super::{Complex, FragmentConstants};
    pub trait Modifier {
        /// Override as necessary
        #[inline(always)]
        fn modify(&self, _z: &mut Complex) {}

        /// Override as necessary
        #[inline(always)]
        fn iterate_algorithm(
            &self,
            constants: &FragmentConstants,
            z: Complex,
            c: Complex,
            iters: u32,
        ) -> Complex {
            self.iterate_algorithm_base(constants, z, c, iters)
        }

        /// Do not override!
        #[inline(always)]
        fn iterate_algorithm_base(
            &self,
            constants: &FragmentConstants,
            z: Complex,
            c: Complex,
            _iters: u32,
        ) -> Complex {
            use shader_common::NumericType;
            match constants.exponent.typ {
                NumericType::Integer if constants.exponent.int == 2 => z * z + c,
                NumericType::Integer => z.powi(constants.exponent.int).to_rectangular() + c,
                NumericType::Float => z.powf(constants.exponent.float).to_rectangular() + c,
            }
        }
    }
}

macro_rules! standard_fractal {
    ($name: ident) => {
        struct $name {
            pub c: Complex,
        }
        impl Fractal for $name {
            fn iterate(&self, constants: &FragmentConstants) -> FractalResult {
                self.iterate_inner(self.c, constants)
            }
        }
    };
}

standard_fractal!(Mandelbrot);
impl private::Modifier for Mandelbrot {}

standard_fractal!(Mandelbar);
impl private::Modifier for Mandelbar {
    // Same as mandelbrot, but conjugate c each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        *z = z.conjugate();
    }
}

standard_fractal!(BurningShip);
impl private::Modifier for BurningShip {
    // Same as mandelbrot, but take abs(re) and abs(im) each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        z.re = z.re.abs();
        z.im = z.im.abs();
    }
}

standard_fractal!(Celtic);
impl private::Modifier for Celtic {
    #[inline(always)]
    fn iterate_algorithm(
        &self,
        constants: &FragmentConstants,
        z: Complex,
        c: Complex,
        iters: u32,
    ) -> Complex {
        // Based on mandelbrot, but using the formula:
        //   z := abs(re(z^2)) + i.im(z^2) + c
        let zz = self.iterate_algorithm_base(constants, z, c, iters);
        Complex {
            // unrolled version (fixed power):
            // re: (z.re * z.re - z.im * z.im).abs() + c.re,
            // im: 2.0 * z.re * z.im + c.im,
            re: zz.re.abs() + c.re,
            im: zz.im + c.im,
        }
    }
}

standard_fractal!(BirdOfPrey);
impl private::Modifier for BirdOfPrey {
    // Same as mandelbrot, but take abs(im) each time
    #[inline(always)]
    fn modify(&self, z: &mut super::Complex) {
        z.im = z.im.abs();
    }
}

standard_fractal!(Variant);
impl private::Modifier for Variant {
    #[inline(always)]
    fn iterate_algorithm(
        &self,
        constants: &FragmentConstants,
        z: Complex,
        c: Complex,
        iters: u32,
    ) -> Complex {
        let zz = self.iterate_algorithm_base(constants, z, c, iters);
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
