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
        // Mandelbar: Same as mandelbrot, but conjugate c each time
        Algorithm::Mandelbar => builder!(Mandelbar { c }).iterations(),
    }
}

/*
 * Mandelbar: Same as mandelbrot, but conjugate c each time
 * Burning Ship: Same as mandelbrot, but abs(z) each time
 * Celtic: Same as mandelbrot, but abs(Re(z)) each time
 * Variant: Same as mandelbrot, but abs(Re(z)) on odd iterations (this one will be challenging to make efficient!)
 * BirdOfPrey: Same as mandelbrot, but abs(Im(z)) each time
 */

pub(crate) trait Fractal: private::Modifier {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult;
    fn iterate_inner(&self, c: Complex, constants: &FragmentConstants) -> FractalResult {
        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let max_iter = constants.max_iter;
        // TODO: Cardoid and period-2 bulb checks in original?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            self.modify(&mut z);
            z = z * z + c;
            iters += 1;
            norm_sqr = z.abs_sq();
        }
        let inside = iters == max_iter && (norm_sqr < ESCAPE_THRESHOLD_SQ);

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // A couple of extra iterations helps decrease the size of the error term
        z = z * z + c;
        z = z * z + c;
        // by the logarithm of a power law,
        // point.value.norm().ln().ln() === (point.value.norm_sqr().ln() * 0.5).ln())
        let smoothed_iters =
            (iters + 2) as f32 - (z.abs_sq().ln() * 0.5).ln() / core::f32::consts::LN_2;

        FractalResult {
            inside,
            iters,
            smoothed_iters,
        }
    }
}

mod private {
    pub trait Modifier {
        #[inline(always)]
        fn modify(&self, _z: &mut super::Complex) {}
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
