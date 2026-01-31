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

#[clippy::format_args]
#[allow(unused_macros)]
macro_rules! xprintln {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "spirv"))]
        eprintln!($($arg)*);
    };
}

use core::marker::PhantomData;

use bytemuck::NoUninit;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{Complex, FragmentConstants, PointResult, Vec2};
use crate::{Algorithm, Flags, exponentiation::Exponentiator};

// *sigh* these are constants are pub(crate) in core
const EXP_MASK_F32: u32 = 0x7F80_0000;
const MAN_MASK_F32: u32 = 0x7F_FFFF;
/// Infinity test lifted from `core::f32`.
///
/// Unfortunately, `f32::is_infinite()` relies on embedded an infinity literal, which the spirv
/// verifier disallows.
///
/// Something similar could be done if we needed `f32::classify`, which relies on u8. Alas, spirv
/// does not guarantee u8 is present.
///
/// So we rawdog it ourselves.
///
/// However, `f32::is_nan()` appears to be GPU-safe.
fn f32_is_infinite(f: f32) -> bool {
    let b = f.to_bits();
    (b & EXP_MASK_F32 == EXP_MASK_F32) && (b & MAN_MASK_F32 == 0)
}

#[macro_export]
/// Exponent dispatcher.
/// **(Second-order macro!)**
///
/// Parameters:
/// * `$exponent`: The exponent to dispatch on
/// * `$run_it`: Target macro that does something useful. Invoked as `$run_it!(Exponentiator,
///   $alg)`.
/// * `$alg`: Algorithm type to pass to `$run_it`
macro_rules! exponent_monomorph {
    ($exponent: expr, $run_it: ident, $alg:ty) => {
        match $exponent.typ {
            $crate::push_constants::NumericType::Integer if $exponent.int == 2 => {
                $run_it!($crate::exponentiation::Power2 {}, $alg)
            }
            $crate::push_constants::NumericType::Integer if $exponent.int == 3 => {
                $run_it!($crate::exponentiation::Power3 {}, $alg)
            }
            $crate::push_constants::NumericType::Integer if $exponent.int == 4 => {
                $run_it!($crate::exponentiation::Power4 {}, $alg)
            }
            $crate::push_constants::NumericType::Integer if $exponent.int == 5 => {
                $run_it!($crate::exponentiation::Power5 {}, $alg)
            }
            $crate::push_constants::NumericType::Integer if $exponent.int == 6 => {
                $run_it!($crate::exponentiation::Power6 {}, $alg)
            }
            $crate::push_constants::NumericType::Integer => {
                $run_it!($crate::exponentiation::IntegerPower($exponent.int), $alg)
            }
            $crate::push_constants::NumericType::Float => {
                $run_it!($crate::exponentiation::RealPower($exponent.real), $alg)
            }
            $crate::push_constants::NumericType::Complex => {
                $run_it!($crate::exponentiation::ComplexPower::from($exponent), $alg)
            } // _ => unreachable!(),
        }
    };
}

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

    macro_rules! run_fractal {
        ($expo:expr,$alg:ty) => {
            Runner {
                frag: constants,
                algorithm: PhantomData::<$alg>,
                consts: RunningConstants {
                    c,
                    dc: dc.into(),
                    modifiers: AlgorithmModifiers::from(constants),
                    exponentiator: $expo,
                    reference_points,
                    n_reference: constants.n_reference_points as usize,
                },
            }
            .run()
        };
    }

    let mut result = if constants.flags.contains(Flags::PERTURBATION_MODE) {
        exponent_monomorph!(constants.exponent, run_fractal, MandelbrotPerturbed)
    } else {
        exponent_monomorph!(constants.exponent, run_fractal, MandelbrotFamily)
    };
    if constants.flags.contains(Flags::ITERATION_CULL) {
        result.cull_iterations();
    }
    result
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
    dc: Complex,
    modifiers: AlgorithmModifiers,
    exponentiator: E,
    /// Reference points (only used in perturbation mode)
    reference_points: &'a [Vec2],
    /// Number of reference points (only used in perturbation mode)
    n_reference: usize,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, NoUninit)]
#[cfg_attr(not(target_arch = "spirv"), derive(strum::Display))]
#[repr(u32)]
pub enum BoundaryClass {
    #[default]
    Indeterminate,
    Inside,
    VeryClose,
    Close,
    NotClose,
}

#[derive(Default, Debug)]
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
    boundary: BoundaryClass,
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

        // distance estimate, angle, radius
        let za = vars.z.abs();
        // special case to avoid hitting a NaN when calculating ln(0)
        let ln_za = if za == 0.0 { 0.0 } else { za.ln() };

        if vars.boundary == BoundaryClass::Indeterminate {
            if iters == self.frag.max_iter {
                vars.boundary = BoundaryClass::Inside;
            } else {
                // abs() overflows on deeper zooms, so use geometry to calculate |dz_dist| a
                // different way
                let arg = vars.dz_dist.re.atan2(vars.dz_dist.im);
                let abs = vars.dz_dist.re / arg.sin();
                let distance = 2.0 * ln_za * za / abs;
                let threshold = self.frag.pixel_spacing() / 4.0;
                if distance <= threshold {
                    vars.boundary = BoundaryClass::Close;
                } else {
                    vars.boundary = BoundaryClass::NotClose;
                }
            }
        }
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
        PointResult::new(iters, smoothed_iters, angle, prev_norm_sqr, vars.boundary)
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
    vars.norm_sqr = z.abs_sq();
    if vars.boundary == BoundaryClass::Indeterminate {
        vars.dz_dist = power * z_in.powf(power - 1.0).to_rectangular() * vars.dz_dist + 1.0;
        if f32_is_infinite(vars.dz_dist.re) || f32_is_infinite(vars.dz_dist.im) {
            vars.boundary = BoundaryClass::VeryClose;
        }
    }
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
        + consts.dc;

    // TODO: Non-2 exponents are not yet verified.
    if vars.boundary == BoundaryClass::Indeterminate {
        vars.dz_dist = 2.0 * vars.z * vars.dz_dist + 1.0;
        if f32_is_infinite(vars.dz_dist.re) || f32_is_infinite(vars.dz_dist.im) {
            vars.boundary = BoundaryClass::VeryClose;
        }
    }

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
    if vars.norm_sqr < dz_p.abs_sq() || vars.ref_iter >= consts.n_reference {
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

    use super::Flags;
    use crate::{
        BigVec2, FragmentConstants, Palette, Size, Vec2,
        enums::Algorithm,
        fractal::{self, BoundaryClass},
        push_constants::{NumericType, PushExponent},
        vec2,
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

    #[test]
    #[allow(clippy::missing_panics_doc, clippy::cast_possible_truncation)]
    fn distance_estimator_boundary_classification() {
        let centre = vec2(-1.5, 0.0);
        let centre_big = BigVec2::try_new(centre.x, centre.y).unwrap();
        let mut consts = FragmentConstants {
            flags: Flags::PERTURBATION_MODE,
            viewport_translate: centre,
            viewport_zoom: 1.0,
            size: Size::new(1000, 1000),
            buffer_size: Size::new(1, 1),
            max_iter: 1000,
            ..Default::default()
        };
        let mut ref_points = Vec::new();
        super::mandelbrot_perturbed_compute_reference_iters(
            &mut ref_points,
            &centre_big,
            Algorithm::Mandelbrot,
            consts.max_iter,
        );
        consts.n_reference_points = ref_points.len() as u32;

        let mut run_case = |i| {
            consts.viewport_zoom = 10.0f32.powi(i);
            let offset = vec2(0.0, 1.0 / consts.viewport_zoom);

            println!(
                "\nzoom step {i}: z={zoom:e} y={y:e}",
                zoom = consts.viewport_zoom,
                y = offset.y,
            );
            let result = super::render(&consts, offset, &ref_points);
            println!("{result:?}");
            result.boundary == BoundaryClass::Close || result.boundary == BoundaryClass::VeryClose
        };
        assert!(!run_case(0));
        assert!(!run_case(15));
        assert!(!run_case(16));
        assert!(!run_case(17));
        assert!(!run_case(34));
        // on f32, this is the point where things start to overflow and go a bit weird
        assert!(run_case(35));
        assert!(!run_case(36)); // sunspot
        assert!(run_case(37));
        assert!(run_case(38));
        assert!(run_case(100));
    }
}
