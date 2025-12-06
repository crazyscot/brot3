//! Fractal algorithms.
//! Can also be called on the host.

#[cfg(not(target_arch = "spirv"))]
const DEBUG_FRACTAL: bool = false;

pub(crate) const ESCAPE_THRESHOLD: f32 = 10.0;
pub(crate) const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
const LOGLOG2_ESCAPE_THRESHOLD: f32 = 1.732_020_9;

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
use crate::exponentiation::{
    ComplexPower, Exponentiator, IntegerPower, Power2, Power3, Power4, Power5, Power6, RealPower,
};
use shader_common::NumericType;

use core::marker::PhantomData;

pub fn render(constants: &FragmentConstants, point: Vec2) -> PointResult {
    use shader_common::enums::Algorithm;
    let c = match constants.algorithm {
        // Mandeldrop is the same as Mandelbrot but with a different c
        Algorithm::Mandeldrop => Complex::from(point).recip(),
        _ => Complex::from(point),
    };

    macro_rules! builder {
        ($fractal:ident, $c_value:expr) => {{
            match constants.exponent.typ {
                NumericType::Integer if constants.exponent.int == 2 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: Power2 {},
                }
                .run(),
                NumericType::Integer if constants.exponent.int == 3 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: Power3 {},
                }
                .run(),
                NumericType::Integer if constants.exponent.int == 4 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: Power4 {},
                }
                .run(),
                NumericType::Integer if constants.exponent.int == 5 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: Power5 {},
                }
                .run(),
                NumericType::Integer if constants.exponent.int == 6 => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: Power6 {},
                }
                .run(),

                NumericType::Integer => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: IntegerPower(constants.exponent.int),
                }
                .run(),

                NumericType::Float => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: RealPower(constants.exponent.real),
                }
                .run(),
                NumericType::Complex => Runner {
                    constants,
                    algo: PhantomData::<$fractal>,
                    c: $c_value,
                    expo: ComplexPower::from(constants.exponent),
                }
                .run(),
                _ => unreachable!(),
            }
        }};
    }
    builder!(MandelbrotFamily, c)
}

struct Runner<'a, F, E>
where
    F: AlgorithmDetail,
    E: Exponentiator,
{
    constants: &'a FragmentConstants,
    algo: PhantomData<F>,
    c: Complex,
    expo: E,
}

/// Having a match expression in a hot loop hurts performance pretty badly,
/// so we're reducing it down to some simple boolean decisions.
#[derive(Default, Copy, Clone)]
pub(crate) struct AlgorithmModifiers {
    iter_re_abs: bool,
    iter_re_variant: bool,

    premod_re_abs: bool,
    premod_im_abs: bool,
    premod_im_conjugate: bool,
}

impl From<&FragmentConstants> for AlgorithmModifiers {
    fn from(consts: &FragmentConstants) -> Self {
        use shader_common::enums::Algorithm;

        let mut rv = AlgorithmModifiers::default();

        /* Point pre-modifiers:
           Default is to do nothing
           Mandelbar takes the complex conjugate of the point i.e. negates z.im
           Burning Ship takes the abs of both parts of z
           Bird of Prey takes the abs of z.im
        */
        match consts.algorithm {
            Algorithm::Mandelbar => {
                rv.premod_im_conjugate = true;
            }
            Algorithm::BirdOfPrey => {
                rv.premod_im_abs = true;
            }
            Algorithm::BurningShip => {
                rv.premod_re_abs = true;
                rv.premod_im_abs = true;
            }
            _ => {}
        }

        /* Algorithm iteration modifiers:
          Celtic applies abs() to z.re before adding c
          Variant applies abs() to z.re before adding c IFF iters is odd.
          Otherwise use z.re unmodified.
        */
        match consts.algorithm {
            Algorithm::Celtic => rv.iter_re_abs = true,
            Algorithm::Variant => rv.iter_re_variant = true,
            _ => (),
        }
        rv
    }
}

impl<F, E> Runner<'_, F, E>
where
    F: AlgorithmDetail,
    E: Exponentiator,
{
    fn run(self) -> PointResult {
        let mut iters = 0;
        let mut z = Complex::ZERO;
        let mut dz = Complex::ZERO;
        let mut prev_z = Complex::ZERO;
        let mut norm_sqr = z.abs_sq();
        let mut prev_norm_sqr = 0.0;
        let max_iter = self.constants.max_iter;

        deprintln!("DBG: run for c={:?}", self.c);
        // TODO: Cardoid and period-2 bulb checks in original?

        let iterate_params = AlgorithmModifiers::from(self.constants);

        while iters < max_iter && norm_sqr < ESCAPE_THRESHOLD_SQ {
            F::pre_modify_point(&mut z, iterate_params);
            prev_z = z;
            prev_norm_sqr = norm_sqr;
            (z, dz) = F::iterate_algorithm(z, dz, self.c, iters, iterate_params, self.expo);
            iters += 1;
            norm_sqr = z.abs_sq();
            deprintln!("DBG: iters={iters}, z={z}, dz={dz}, |z|^2={norm_sqr}");
        }

        // distance estimate, angle
        let za = z.abs();
        // special case to avoid hitting a NaN when calculating ln(0)
        let ln_za = if za == 0.0 { 0.0 } else { za.ln() };

        let distance = 2.0 * ln_za * za / dz.abs();
        deprintln!("za {za} zaln {ln_za} dzabs {} dist {distance}", dz.abs());
        let angle = prev_z.arg();
        let radius_sqr = prev_norm_sqr;

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // The log(exponent) term is necessary for powers other than 2.
        // Note that the log of theexponent is not allowed to be 0 or subnormal (we divide by
        // it below), so we special case those regions (in Exponentiator).

        let z_abs_sq = z.abs_sq();
        // take two logs, avoiding NaN
        let log_log_zn = if z_abs_sq <= 1.0 {
            // special case: log2(log2(1+epsilon)) tends to -inf
            -1000.0
        } else {
            // by the logarithm of a power law,
            // z.norm().log() === z.norm_sqr().log() * 0.5
            (z_abs_sq.log2() * 0.5).log2()
        };

        let smoothed_iters = 1. + LOGLOG2_ESCAPE_THRESHOLD - log_log_zn / self.expo.log2();

        // sigh! saturating_add is not currently implemented, so do it ourselves:
        let inside = norm_sqr < ESCAPE_THRESHOLD_SQ;
        iters = if inside { u32::MAX } else { iters };
        PointResult::new_outside(iters, smoothed_iters, distance, angle, radius_sqr)
    }
}

pub(crate) trait AlgorithmDetail {
    /// Pre-modifies a point before applying the algorithm.
    ///
    /// Override as necessary.
    #[inline(always)]
    fn pre_modify_point(_z: &mut Complex, _params: AlgorithmModifiers) {}

    /// One iteration of the fractal algorithm.
    ///
    /// The provided implementation computes `z := z.pow(e) + c`, but this doesn't
    /// suit all algorithms. Override as necessary.
    fn iterate_algorithm<E: Exponentiator>(
        z: Complex,
        dz: Complex,
        c: Complex,
        iters: u32,
        params: AlgorithmModifiers,
        expo: E,
    ) -> (Complex /*z*/, Complex /*dz*/);
}

// returns (z, dz)
fn mandelbrot_family_iterate_algorithm<E: Exponentiator>(
    z: Complex,
    dz: Complex,
    c: Complex,
    iters: u32,
    par: AlgorithmModifiers,
    expo: E,
) -> (Complex, Complex) {
    let power = expo.power();
    let dz = power * z.powf(power - 1.0).to_rectangular() * dz + 1.0;
    let mut z = expo.apply_to(z);
    // Algorithm difference here:
    // Celtic uses z_re_abs instead of z_re.
    // Variant may or may not take z_re_abs depending on the iters count.
    let z_re = z.re;
    let z_re_abs = z_re.abs();
    let is_odd = !iters.is_multiple_of(2);

    let use_z_re_abs = par.iter_re_abs || (par.iter_re_variant && is_odd);

    // Now bring it all together:
    z.re = if use_z_re_abs { z_re_abs } else { z_re };
    (z + c, dz)
}

fn mandelbrot_family_pre_modify_point(z: &mut super::Complex, params: AlgorithmModifiers) {
    let abs_im = z.im.abs();
    let conj_im = -z.im;

    // Algorithm differences here:
    // Mandelbar conjugates i.e. negates the imaginary part
    // BirdOfPrey applies abs() to the imaginary part
    // Burning Ship applies abs() to both parts

    if params.premod_re_abs {
        z.re = z.re.abs();
    }
    z.im = if params.premod_im_abs {
        abs_im
    } else if params.premod_im_conjugate {
        conj_im
    } else {
        z.im
    };
}

struct MandelbrotFamily {}
impl AlgorithmDetail for MandelbrotFamily {
    #[inline(always)]
    fn pre_modify_point(z: &mut Complex, params: AlgorithmModifiers) {
        mandelbrot_family_pre_modify_point(z, params);
    }

    #[inline(always)]
    fn iterate_algorithm<E: Exponentiator>(
        z: Complex,
        dz: Complex,
        c: Complex,
        iters: u32,
        params: AlgorithmModifiers,
        expo: E,
    ) -> (Complex /*z*/, Complex /*dz*/) {
        mandelbrot_family_iterate_algorithm(z, dz, c, iters, params, expo)
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
            buffer_size: Size::new(1, 1),
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

    #[test]
    fn variant_correctness() {
        let point = crate::vec2(-0.75, 0.75);
        let mut consts = test_frag_consts();
        consts.algorithm = Algorithm::Variant;
        // Variant has a debug_assert! consistency check
        let result = fractal::render(&consts, point);
        eprintln!("{result:?}");
    }
}
