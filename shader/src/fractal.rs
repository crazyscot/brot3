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
    }
}

pub(crate) trait Fractal {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult;
}

/// The original Mandelbrot set and its inverse, the Mandeldrop
struct Mandelbrot {
    pub c: Complex,
}

impl Fractal for Mandelbrot {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult {
        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let c = self.c;
        let mut norm_sqr = z.abs_sq();
        let max_iter = constants.max_iter;
        // TODO: Cardoid and period-2 bulb checks in original? Would need to think about the mandeldrop.

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
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

struct Mandelbar {
    pub c: Complex,
}

impl Fractal for Mandelbar {
    fn iterate(&self, constants: &FragmentConstants) -> FractalResult {
        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;

        let mut iters = 0;
        let mut z = Complex::ZERO;
        let c = self.c;
        let mut norm_sqr = z.abs_sq();
        let max_iter = constants.max_iter;

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < max_iter {
            z = z.conjugate(); // Conjugate step
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
