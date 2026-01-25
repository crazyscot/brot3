//! Fractal algorithms.
//! Can also be called on the host.

#![allow(missing_docs)]

#[cfg(not(target_arch = "spirv"))]
const DEBUG_FRACTAL: bool = false;

pub(crate) use crate::push_constants::{ESCAPE_THRESHOLD_SQ, LOGLOG2_ESCAPE_THRESHOLD};

#[clippy::format_args]
macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "spirv"))]
        if DEBUG_FRACTAL {
            eprintln!($($arg)*);
        }
    };
}

use core::marker::PhantomData;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Complex, FragmentConstants, PointResult, Vec2};
use crate::{
    Algorithm, Flags,
    exponentiation::{
        ComplexPower, Exponentiator, IntegerPower, Power2, Power3, Power4, Power5, Power6,
        RealPower,
    },
    push_constants::NumericType,
};

#[must_use]
pub fn render(
    constants: &FragmentConstants,
    offset: Vec2,
    reference_points: &[Vec2],
) -> PointResult {
    let point = offset + constants.viewport_translate;
    let (c, dc) = match constants.algorithm {
        Algorithm::Mandeldrop => {
            // Mandeldrop is the same as Mandelbrot but with a different c.
            // TODO: Need to run the maths properly for perturbation Mandeldrop, understand how to
            // compute dc.
            (Complex::from(point).recip(), offset)
        }
        _ => (Complex::from(point), offset),
    };

    macro_rules! run_it {
        ($expo:expr,$alg:ty) => {
            Runner {
                frag: constants,
                algorithm: PhantomData::<$alg>,
                consts: RunningConstants {
                    c,
                    dc,
                    modifiers: AlgorithmModifiers::from(constants),
                    exponentiator: $expo,
                    reference_points,
                    n_reference: constants.n_reference_points as usize,
                },
            }
            .run()
        };
    }

    macro_rules! build_alg {
        ($alg:ty) => {
            match constants.exponent.typ {
                NumericType::Integer if constants.exponent.int == 2 => run_it!(Power2 {}, $alg),
                NumericType::Integer if constants.exponent.int == 3 => run_it!(Power3 {}, $alg),
                NumericType::Integer if constants.exponent.int == 4 => run_it!(Power4 {}, $alg),
                NumericType::Integer if constants.exponent.int == 5 => run_it!(Power5 {}, $alg),
                NumericType::Integer if constants.exponent.int == 6 => run_it!(Power6 {}, $alg),
                NumericType::Integer => run_it!(IntegerPower(constants.exponent.int), $alg),
                NumericType::Float => run_it!(RealPower(constants.exponent.real), $alg),
                NumericType::Complex => run_it!(ComplexPower::from(constants.exponent), $alg),
                // _ => unreachable!(),
            }
        };
    }
    if constants.flags.contains(Flags::PERTURBATION_MODE) {
        build_alg!(MandelbrotPerturbed)
    } else {
        build_alg!(MandelbrotFamily)
    }
}

struct Runner<'a, F, E>
where
    F: AlgorithmDetail<'a, E>,
    E: Exponentiator,
{
    frag: &'a FragmentConstants,
    algorithm: PhantomData<F>,
    consts: RunningConstants<'a, E>,
}

/// This struct is created once for each [`Runner`] and is constant for that run.
struct RunningConstants<'a, E>
where
    E: Exponentiator,
{
    /// Absolute complex address of the point we are rendering
    c: Complex,
    /// Relative complex address of the point we are rendering (relative to the centre of the
    /// viewport). Used only in perturbation mode.
    dc: Vec2,
    modifiers: AlgorithmModifiers,
    exponentiator: E,
    /// Reference points (only used in perturbation mode)
    reference_points: &'a [Vec2],
    /// Number of reference points (only used in perturbation mode)
    n_reference: usize,
}

#[derive(Default)]
/// These are the Runner variables that `iterate_algorithm()` is expected to keep up to date.
///
/// N.B. that the iteration count is not here; it not a constant either, but `iterate_algorithm`
/// may not modify it.
struct RunningVariables {
    z: Complex,
    dz_dist: Complex,
    norm_sqr: f32,
    dz_perturb: Complex,
    ref_iter: usize,
}

/// Having a match expression in a hot loop hurts performance pretty badly,
/// so we're reducing it down to some simple boolean decisions.
#[derive(Default, Copy, Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct AlgorithmModifiers {
    pub iter_re_abs: bool,
    pub iter_re_variant: bool,

    premod_re_abs: bool,
    premod_im_abs: bool,
    premod_im_conjugate: bool,
}

impl From<&FragmentConstants> for AlgorithmModifiers {
    fn from(consts: &FragmentConstants) -> Self {
        AlgorithmModifiers::from(consts.algorithm)
    }
}

impl From<Algorithm> for AlgorithmModifiers {
    fn from(algorithm: Algorithm) -> Self {
        let mut rv = AlgorithmModifiers::default();

        /* Point pre-modifiers:
           Default is to do nothing
           Mandelbar takes the complex conjugate of the point i.e. negates z.im
           Burning Ship takes the abs of both parts of z
           Bird of Prey takes the abs of z.im
        */
        match algorithm {
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
        match algorithm {
            Algorithm::Celtic => rv.iter_re_abs = true,
            Algorithm::Variant => rv.iter_re_variant = true,
            _ => (),
        }
        rv
    }
}

impl<'a, F, E> Runner<'a, F, E>
where
    F: AlgorithmDetail<'a, E>,
    E: Exponentiator,
{
    fn run(self) -> PointResult {
        let mut iters = 0;
        let mut vars = RunningVariables::default();

        let mut prev_z = Complex::ZERO;
        let mut prev_norm_sqr = 0.0;

        deprintln!("DBG: run for c={:?}", self.consts.c);
        // TODO: Cardoid and period-2 bulb checks in original?

        //let iterate_params = AlgorithmModifiers::from(self.constants);

        while iters < self.frag.max_iter && vars.norm_sqr < ESCAPE_THRESHOLD_SQ {
            F::pre_modify_point(&self.consts, &mut vars);
            prev_z = vars.z;
            prev_norm_sqr = vars.norm_sqr;
            F::iterate_algorithm(&self.consts, &mut vars, iters);
            iters += 1;
            deprintln!(
                "DBG: iters={iters}, z={z}, dz_dist={dz_dist}, |z|^2={norm_sqr}",
                z = vars.z,
                dz_dist = vars.dz_dist,
                norm_sqr = vars.norm_sqr,
            );
        }

        // distance estimate, angle
        let za = vars.z.abs();
        // special case to avoid hitting a NaN when calculating ln(0)
        let ln_za = if za == 0.0 { 0.0 } else { za.ln() };

        let distance = 2.0 * ln_za * za / vars.dz_dist.abs();
        deprintln!(
            "za {za} zaln {ln_za} dzabs {} dist {distance}",
            vars.dz_dist.abs()
        );
        let angle = prev_z.arg();
        let norm_sqr = vars.norm_sqr;

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // The log(exponent) term is necessary for powers other than 2.
        // Note that the log of theexponent is not allowed to be 0 or subnormal (we divide by
        // it below), so we special case those regions (in Exponentiator).

        // take two logs, avoiding NaN
        let log_log_zn = if norm_sqr <= 1.0 {
            // special case: log2(log2(1+epsilon)) tends to -inf
            -1000.0
        } else {
            // by the logarithm of a power law,
            // z.norm().log() === z.norm_sqr().log() * 0.5
            (norm_sqr.log2() * 0.5).log2()
        };

        let smoothed_iters =
            1. + LOGLOG2_ESCAPE_THRESHOLD - log_log_zn / self.consts.exponentiator.log2();

        // sigh! saturating_add is not currently implemented, so do it ourselves:
        let inside = norm_sqr < ESCAPE_THRESHOLD_SQ;
        iters = if inside { u32::MAX } else { iters };
        PointResult::new_outside(iters, smoothed_iters, distance, angle, prev_norm_sqr)
    }
}

trait AlgorithmDetail<'a, E: Exponentiator> {
    /// Pre-modifies a point before applying the algorithm.
    ///
    /// Override as necessary.
    #[inline(always)]
    fn pre_modify_point(_consts: &RunningConstants<'a, E>, _vars: &mut RunningVariables) {}

    /// One iteration of the fractal algorithm.
    ///
    /// The provided implementation computes `z := z.pow(e) + c`, but this doesn't
    /// suit all algorithms. Override as necessary.
    fn iterate_algorithm(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables, iters: u32);
}

fn mandelbrot_family_iterate_algorithm<E: Exponentiator>(
    consts: &RunningConstants<'_, E>,
    vars: &mut RunningVariables,
    iters: u32,
) {
    let params = &consts.modifiers;
    let exponent = &consts.exponentiator;
    let power = exponent.power();
    let z_in = vars.z;

    let dz_dist = power * z_in.powf(power - 1.0).to_rectangular() * vars.dz_dist + 1.0;

    // Raise z to the given power ...
    let mut z = exponent.apply_to(z_in);

    // Algorithm difference here:
    // Celtic uses z_re_abs instead of z_re.
    // Variant may or may not take z_re_abs depending on the iters count.
    let z_re = z.re;
    let z_re_abs = z_re.abs();
    let is_odd = !iters.is_multiple_of(2);

    let use_z_re_abs = params.iter_re_abs || (params.iter_re_variant && is_odd);
    z.re = if use_z_re_abs { z_re_abs } else { z_re };

    // ... and add the constant value
    z += consts.c;

    // Send output
    vars.z = z;
    vars.dz_dist = dz_dist;
    vars.norm_sqr = z.abs_sq();
}

fn mandelbrot_family_pre_modify_point<E: Exponentiator>(
    consts: &RunningConstants<'_, E>,
    vars: &mut RunningVariables,
) {
    mandelbrot_family_pre_modify_point_inner(&mut vars.z, consts.modifiers);
}

/// TODO: Someday, deduplicate this with `mandelbrot_family_pre_modify_point_inner_big`?
fn mandelbrot_family_pre_modify_point_inner(z: &mut Complex, params: AlgorithmModifiers) {
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

#[cfg(not(target_arch = "spirv"))]
/// Pre modification is exposed so that ui can use it.
///
/// TODO: Someday, deduplicate this with `mandelbrot_family_pre_modify_point_inner`?
pub fn mandelbrot_family_pre_modify_point_inner_big(
    z: &mut crate::BigComplex,
    params: AlgorithmModifiers,
) {
    use dashu::base::Abs;
    // TODO: Perturbation mode is not yet verified.

    // Algorithm differences here:
    // Mandelbar conjugates i.e. negates the imaginary part
    // BirdOfPrey applies abs() to the imaginary part
    // Burning Ship applies abs() to both parts

    if params.premod_re_abs {
        z.x = std::mem::take(&mut z.x).abs();
    }
    if params.premod_im_abs {
        z.y = std::mem::take(&mut z.y).abs();
    }
    if params.premod_im_conjugate {
        z.y *= dashu::float::FBig::NEG_ONE;
    }
}

struct MandelbrotFamily {}
impl<'a, E: Exponentiator> AlgorithmDetail<'a, E> for MandelbrotFamily {
    fn pre_modify_point(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables) {
        mandelbrot_family_pre_modify_point(consts, vars);
    }

    fn iterate_algorithm(
        consts: &RunningConstants<'a, E>,
        vars: &mut RunningVariables,
        iters: u32,
    ) {
        mandelbrot_family_iterate_algorithm(consts, vars, iters);
    }
}

struct MandelbrotPerturbed {}
impl<'a, E: Exponentiator> AlgorithmDetail<'a, E> for MandelbrotPerturbed {
    fn pre_modify_point(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables) {
        mandelbrot_family_pre_modify_point(consts, vars);
    }

    fn iterate_algorithm(
        consts: &RunningConstants<'a, E>,
        vars: &mut RunningVariables,
        iters: u32,
    ) {
        mandelbrot_perturbed_iterate_algorithm(consts, vars, iters);
    }
}

fn mandelbrot_perturbed_iterate_algorithm<E: Exponentiator>(
    consts: &RunningConstants<'_, E>,
    vars: &mut RunningVariables,
    _iter: u32,
) {
    // References: https://mrob.com/pub/muency/bivariatelinearapproximati.html
    // https://fractalforums.org/fractal-mathematics-and-new-theories/28/another-solution-to-perturbation-glitches/4360

    // let params = &consts.modifiers;
    // let exponent = &consts.exponentiator;
    // let power = exponent.power();
    let mut dz_p = vars.dz_perturb;

    // Here's our old friend z := z^n + c, in perturbation form:
    // TODO: Do the maths for non-2 exponents.
    dz_p = 2.0 * dz_p * Complex::from(consts.reference_points[vars.ref_iter])
        + dz_p * dz_p
        + Complex::from(consts.dc);

    // We'll add dc in just a moment.
    // TODO: Non-2 exponents are not yet verified.
    vars.dz_dist = 2.0 * vars.z * vars.dz_dist + 1.0;

    // TODO: Need to run the maths properly for the other algorithms and how they differ.
    /*
    // Algorithm difference here:
    // Celtic uses z_re_abs instead of z_re.
    // Variant may or may not take z_re_abs depending on the iters count.
    let zz_re = dz_p.re;
    let zz_re_abs = zz_re.abs();
    let is_odd = !iter.is_multiple_of(2);
    let use_z_re_abs = params.iter_re_abs || (params.iter_re_variant && is_odd);
    dz_p.re = if use_z_re_abs { zz_re_abs } else { zz_re };
    */

    // Now we can compute this iteration's value of `z`
    vars.ref_iter += 1;
    let z = Complex::from(consts.reference_points[vars.ref_iter]) + dz_p;
    vars.norm_sqr = z.abs_sq();
    if vars.norm_sqr < dz_p.abs_sq() || vars.ref_iter == consts.n_reference {
        dz_p = z;
        vars.ref_iter = 0;
    }
    vars.z = z;
    vars.dz_perturb = dz_p;
}

/// Updates a vector of reference points.
///
/// The vector will be cleared and rewritten.
#[cfg(not(target_arch = "spirv"))]
#[allow(clippy::missing_panics_doc, reason = "it's a const conversion")]
pub fn mandelbrot_perturbed_compute_reference_iters(
    points: &mut Vec<Vec2>,
    centre: &crate::BigVec2,
    algorithm: Algorithm,
    max_iter: u32,
) {
    use dashu::base::Sign;
    use dashu_float::{FBig, round::mode as RoundingMode};

    use crate::BigComplex;

    points.clear();
    let modifiers = AlgorithmModifiers::from(algorithm);
    let threshold_sq = FBig::<RoundingMode::Zero>::try_from(ESCAPE_THRESHOLD_SQ).unwrap();

    // In perturbation mode, we always use the centre of the viewport as the reference
    // iteration.
    let mut ctemp = BigComplex::from(centre.clone());
    if algorithm == Algorithm::Mandeldrop {
        ctemp = ctemp.recip();
    }
    let c = ctemp;
    let mut z = BigComplex::ZERO.with_precision(c.precision_larger()); // TODO: What precision do we need?
    points.push(Vec2::ZERO);
    let mut iter = 0;

    while iter < max_iter && z.norm_squared() < threshold_sq {
        mandelbrot_family_pre_modify_point_inner_big(&mut z, modifiers);

        // <<< This is iterate_algorithm
        // TODO: Implement powers other than 2 (requires support in BigComplex)
        z = z.square();

        // Algorithm difference here:
        // Celtic uses z_re_abs instead of z_re.
        // Variant may or may not take z_re_abs depending on the iters count.
        let is_odd = !iter.is_multiple_of(2);
        let use_z_re_abs = modifiers.iter_re_abs || (modifiers.iter_re_variant && is_odd);
        if use_z_re_abs && z.x.sign() == Sign::Negative {
            z.x *= Sign::Negative;
        }
        z = z + &c;
        // >>> End of iterate_algorithm analogue

        iter += 1;
        points.push(z.as_vec2());
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use const_default::ConstDefault as _;
    use pretty_assertions::assert_eq;

    use super::{Flags, NumericType};
    use crate::{
        FragmentConstants, Palette, Size, Vec2, enums::Algorithm, fractal,
        push_constants::PushExponent, vec2,
    };

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
            n_reference_points: 0,
        }
    }

    #[test]
    fn mandelbrot_known_answer() {
        #![allow(clippy::float_cmp)]
        let point = crate::vec2(-0.75, 0.75);
        eprintln!("{:#?}", test_frag_consts());
        let result = fractal::render(
            &test_frag_consts(),
            point - test_frag_consts().viewport_translate,
            &[Vec2::ZERO; 0],
        );
        eprintln!("{result:?}");
        assert_eq!(result.iters_fraction(), 0.522_014_14);
    }

    #[test]
    fn mandelbrot_known_answer_cpow() {
        #![allow(clippy::float_cmp)]
        let point = crate::vec2(-0.75, 0.75);
        let mut consts = test_frag_consts();
        consts.exponent.typ = NumericType::Complex;
        consts.exponent.real = 2.0;
        consts.exponent.imag = 0.0;
        eprintln!("{consts:#?}");
        let result = fractal::render(&consts, point - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("{result:?}");
        assert_eq!(result.iters_fraction(), 0.522_014_6);
    }

    #[test]
    fn variant_correctness() {
        let point = crate::vec2(-0.75, 0.75);
        let mut consts = test_frag_consts();
        consts.algorithm = Algorithm::Variant;
        // Variant has a debug_assert! consistency check
        let result = fractal::render(&consts, point - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("{result:?}");
    }
}
