//! Fractal algorithm implementations
//! Can also be called on the host.
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

#[cfg(not(spirv))]
const DEBUG_FRACTAL: bool = false;

pub(crate) use crate::{ESCAPE_THRESHOLD_LOGLOG2, ESCAPE_THRESHOLD_SQ};

#[clippy::format_args]
macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(spirv))]
        if DEBUG_FRACTAL {
            eprintln!($($arg)*);
        }
    };
}

#[clippy::format_args]
#[allow(unused_macros)]
macro_rules! xprintln {
    ($($arg:tt)*) => {
        #[cfg(not(spirv))]
        eprintln!($($arg)*);
    };
}

use core::marker::PhantomData;

#[cfg(spirv)]
use crate::Real;
use crate::{
    Complex, Vec2,
    data::{Algorithm, BoundaryClass, Flags, FragmentConstants, PointResult},
    maths::Exponentiator,
};

// *sigh* these are constants are pub(crate) in core
const EXP_MASK_F32: u32 = 0x7F80_0000;
const MAN_MASK_F32: u32 = 0x7F_FFFF;
/// Infinity test lifted from `core::f32`.
///
/// Unfortunately, `f32::is_infinite()` relies on an embedded infinity literal, which the spirv
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
            $crate::data::NumericType::Integer if $exponent.int == 2 => {
                $run_it!($crate::maths::Power2 {}, $alg)
            }
            $crate::data::NumericType::Integer if $exponent.int == 3 => {
                $run_it!($crate::maths::Power3 {}, $alg)
            }
            $crate::data::NumericType::Integer if $exponent.int == 4 => {
                $run_it!($crate::maths::Power4 {}, $alg)
            }
            $crate::data::NumericType::Integer if $exponent.int == 5 => {
                $run_it!($crate::maths::Power5 {}, $alg)
            }
            $crate::data::NumericType::Integer if $exponent.int == 6 => {
                $run_it!($crate::maths::Power6 {}, $alg)
            }
            $crate::data::NumericType::Integer => {
                $run_it!(
                    $crate::maths::RealPower(<_ as $crate::easy_cast::Cast<f32>>::cast(
                        $exponent.int
                    )),
                    $alg
                )
            }
            $crate::data::NumericType::Float => {
                $run_it!($crate::maths::RealPower($exponent.real), $alg)
            }
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

    let (c, dc) = if cfg!(feature = "all-fractals") && constants.algorithm == Algorithm::Mandeldrop
    {
        (Complex::from(point).recip(), offset)
    } else {
        (Complex::from(point), offset)
    };

    macro_rules! run_fractal {
        ($expo:expr,$alg:ty) => {
            Runner {
                frag: constants,
                algorithm: PhantomData::<$alg>,
                consts: RunningConstants {
                    c,
                    dc: dc.into(),
                    algorithm: constants.algorithm,
                    #[cfg(feature = "all-fractals")]
                    modifiers: AlgorithmModifiers::from(constants),
                    exponentiator: $expo,
                    reference_points,
                    n_reference: reference_points.len(),
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
#[allow(missing_debug_implementations)] // pub only in cfg(test)
#[derive(Clone, Copy)]
pub struct RunningConstants<'a, E>
where
    E: Exponentiator,
{
    /// Absolute complex address of the point we are rendering
    c: Complex,
    /// Relative complex address of the point we are rendering (relative to the centre of the
    /// viewport). Used only in perturbation mode.
    dc: Complex,
    algorithm: Algorithm,
    #[cfg(feature = "all-fractals")]
    modifiers: AlgorithmModifiers,
    exponentiator: E,
    /// Reference points (only used in perturbation mode)
    #[doc(hidden)]
    pub reference_points: &'a [Vec2],
    /// Number of reference points (only used in perturbation mode)
    #[doc(hidden)]
    pub n_reference: usize,
}

impl<E: Exponentiator> RunningConstants<'_, E> {
    #[doc(hidden)]
    pub fn standard_with(c: Complex, exponentiator: E, algorithm: Algorithm) -> Self {
        Self {
            c,
            dc: Complex::ZERO,
            algorithm,
            #[cfg(feature = "all-fractals")]
            modifiers: AlgorithmModifiers::from(algorithm),
            exponentiator,
            reference_points: &[],
            n_reference: 0,
        }
    }

    #[doc(hidden)]
    pub fn perturbed_with(
        c: Complex,
        exponentiator: E,
        algorithm: Algorithm,
        dc: Complex,
    ) -> RunningConstants<'static, E> {
        RunningConstants {
            c,
            dc,
            algorithm,
            #[cfg(feature = "all-fractals")]
            modifiers: AlgorithmModifiers::from(algorithm),
            exponentiator,
            reference_points: &[],
            n_reference: 0,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
/// These are the Runner variables that `iterate_algorithm()` is expected to keep up to date.
///
/// N.B. that the iteration count is not here; it not a constant either, but `iterate_algorithm`
/// may not modify it.
#[allow(missing_debug_implementations)]
pub struct RunningVariables {
    z: Complex,
    dz_dist: Complex,
    #[doc(hidden)]
    pub norm_sqr: f32,
    dz_perturb: Complex,
    ref_iter: usize,
    boundary: BoundaryClass,
}

/// Having a match expression in a hot loop hurts performance pretty badly,
/// so we're reducing it down to some simple boolean decisions.
#[cfg(feature = "all-fractals")]
#[derive(Default, Copy, Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct AlgorithmModifiers {
    pub iter_re_abs: bool,
    pub iter_re_variant: bool,

    premod_re_abs: bool,
    premod_im_abs: bool,
    premod_im_conjugate: bool,
}

#[cfg(feature = "all-fractals")]
impl From<&FragmentConstants> for AlgorithmModifiers {
    fn from(consts: &FragmentConstants) -> Self {
        AlgorithmModifiers::from(consts.algorithm)
    }
}

#[cfg(feature = "all-fractals")]
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
        if !self.frag.flags.contains(Flags::DISTANCE_ESTIMATE) {
            vars.boundary = BoundaryClass::Ignored;
        }

        let mut prev_z = Complex::ZERO;
        let mut prev_norm_sqr = 0.0;

        deprintln!("DBG: run for c={:?}", self.consts.c);

        // Power-zero short-circuit: for f(z) = z^0 + c (no modifiers), the orbit is:
        //   z₀=0, z₁=c, z₂=1+c, z₃=1+c, ...  (fixed point from iteration 2 onward)
        // This is analytically solvable; no iteration loop is needed.
        // `power()` is a push constant → the outer check is a *uniform* branch (no warp
        // divergence cost). Only valid when no algorithm modifiers alter the orbit.
        #[cfg(feature = "DISABLED_power_zero_short_circuit")]
        #[allow(clippy::float_cmp)]
        if self.consts.exponentiator.power() == 0.0
            && self.consts.algorithm == Algorithm::Mandelbrot
        {
            let fixed_point = Complex::ONE + self.consts.c;
            let fp_norm_sq = fixed_point.abs_sq();
            if fp_norm_sq >= ESCAPE_THRESHOLD_SQ {
                // Orbit escapes at iteration 2.
                // prev_z=c, prev_norm_sqr=|c|² (values just before the escaping iteration).
                let log_log_zn = (fp_norm_sq.log2() * 0.5).log2();
                let smoothed =
                    1.0 + ESCAPE_THRESHOLD_LOGLOG2 - log_log_zn / self.consts.exponentiator.log2();
                return PointResult::new(
                    2,
                    smoothed,
                    self.consts.c.arg(),
                    self.consts.c.abs_sq(),
                    BoundaryClass::Ignored,
                );
            }
            return PointResult::new(u32::MAX, 0.0, 0.0, 0.0, BoundaryClass::Inside);
        }

        // Cardioid and period-2 bulb short-circuit for standard Mandelbrot (power=2, no
        // modifiers). Points in these regions are analytically guaranteed to be inside the
        // set; no iteration is required at all.
        // The exponent check is a uniform branch; the per-pixel geometric tests are cheap
        // arithmetic that avoids running max_iter iterations for a large fraction of pixels.
        //   Main cardioid:    q·(q + (re-¼)) ≤ ¼·im²,  where q = (re-¼)² + im²
        //   Period-2 bulb:    (re+1)² + im² < 1/16
        #[allow(clippy::float_cmp)]
        if self.consts.exponentiator.power() == 2.0
            && self.consts.algorithm == Algorithm::Mandelbrot
        {
            let c = self.consts.c;
            let cr14 = c.re - 0.25;
            let im2 = c.im * c.im;
            let q = cr14 * cr14 + im2;
            let cr1 = c.re + 1.0;
            if q * (q + cr14) <= 0.25 * im2 || cr1 * cr1 + im2 < 0.0625 {
                return PointResult::new(u32::MAX, 0.0, 0.0, 0.0, BoundaryClass::Inside);
            }
        }

        while iters < self.frag.max_iter && vars.norm_sqr < ESCAPE_THRESHOLD_SQ {
            #[cfg(feature = "all-fractals")]
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
        // Branchless ln(za): when za==0, iters==max_iter (escape requires norm_sqr >= threshold),
        // so ln_za is never used in the final boundary result; .max() keeps all lanes finite
        // and avoids the 0*(-inf)=NaN that the old conditional guarded against.
        let ln_za = za.max(f32::MIN_POSITIVE).ln();

        // This section used to be three nested branches, which caused warp divergence on GPU.
        // This way round, all lanes execute the same instructions and there is no warp divergence.
        //
        // Lanes with boundary==VeryClose or Ignored have their computed indeterminate_class
        // unconditionally discarded by the outermost select, so computing it for them is safe.

        // abs() overflows on deeper zooms, so use geometry to calculate |dz_dist| differently.
        if vars.boundary == BoundaryClass::Indeterminate {
            let abs_dz = vars.dz_dist.abs();
            let distance = 2.0 * ln_za * za / abs_dz;
            let threshold = self.frag.pixel_spacing() / 4.0;

            let dist_class = if distance <= threshold {
                BoundaryClass::Close
            } else {
                BoundaryClass::NotClose
            };
            vars.boundary = if iters == self.frag.max_iter {
                BoundaryClass::Inside
            } else {
                dist_class
            };
        }
        let angle = prev_z.arg();
        let norm_sqr = vars.norm_sqr;

        // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
        // The log(exponent) term is necessary for powers other than 2.
        // Note that the log of the exponent is not allowed to be 0 or subnormal (we divide by
        // it below), so we special case those regions (in Exponentiator).

        // take two logs, avoiding NaN.
        // by the logarithm of a power law,
        // z.norm().log() === z.norm_sqr().log() * 0.5
        let log_log_zn = (norm_sqr.max(1.0 + f32::MIN_POSITIVE).log2() * 0.5).log2();

        let smoothed_iters =
            1. + ESCAPE_THRESHOLD_LOGLOG2 - log_log_zn / self.consts.exponentiator.log2();

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
    #[cfg(feature = "all-fractals")]
    fn pre_modify_point(_consts: &RunningConstants<'a, E>, _vars: &mut RunningVariables) {}

    /// One iteration of the fractal algorithm.
    ///
    /// The provided implementation computes `z := z.pow(e) + c`, but this doesn't
    /// suit all algorithms. Override as necessary.
    fn iterate_algorithm(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables, iters: u32);
}

#[inline]
pub fn mandelbrot_family_iterate_algorithm<E: Exponentiator>(
    consts: &RunningConstants<'_, E>,
    vars: &mut RunningVariables,
    #[allow(unused_variables)] iters: u32,
) {
    let power = consts.exponentiator.power();
    let z_in = vars.z;

    // Raise z to the given power ...
    let mut z = consts.exponentiator.apply_to(z_in);

    #[cfg(feature = "all-fractals")]
    {
        let params = &consts.modifiers;
        // Algorithm difference here:
        // Celtic uses z_re_abs instead of z_re.
        // Variant may or may not take z_re_abs depending on the iters count.
        let z_re = z.re;
        let z_re_abs = z_re.abs();
        let is_odd = !iters.is_multiple_of(2);

        let use_z_re_abs = params.iter_re_abs || (params.iter_re_variant && is_odd);
        z.re = if use_z_re_abs { z_re_abs } else { z_re };
    }

    // ... and add the constant value
    z += consts.c;

    // Send output
    vars.z = z;
    vars.norm_sqr = z.abs_sq();
    if vars.boundary == BoundaryClass::Indeterminate {
        vars.dz_dist =
            consts.exponentiator.apply_power_minus_1_to(z_in) * vars.dz_dist * power + 1.0;
        if f32_is_infinite(vars.dz_dist.re) || f32_is_infinite(vars.dz_dist.im) {
            vars.boundary = BoundaryClass::VeryClose;
        }
    }
}

#[inline]
#[cfg(feature = "all-fractals")]
fn mandelbrot_family_pre_modify_point<E: Exponentiator>(
    consts: &RunningConstants<'_, E>,
    vars: &mut RunningVariables,
) {
    mandelbrot_family_pre_modify_point_inner(&mut vars.z, consts.modifiers);
}

#[cfg(feature = "all-fractals")]
/// TODO: Someday, deduplicate this with `mandelbrot_family_pre_modify_point_inner_big`?
#[inline]
fn mandelbrot_family_pre_modify_point_inner(z: &mut Complex, params: AlgorithmModifiers) {
    let abs_im = z.im.abs();
    let conj_im = -z.im;

    // Algorithm differences here:
    // Mandelbar conjugates i.e. negates the imaginary part
    // BirdOfPrey applies abs() to the imaginary part
    // Burning Ship applies abs() to both parts

    let zr_abs = z.re.abs();
    z.re = if params.premod_re_abs { zr_abs } else { z.re };
    z.im = if params.premod_im_abs {
        abs_im
    } else if params.premod_im_conjugate {
        conj_im
    } else {
        z.im
    };
}

#[cfg(not(spirv))]
/// Part of the high-precision perturbation-mode calculations on CPU.
///
/// TODO: Someday, deduplicate this with `mandelbrot_family_pre_modify_point_inner`?
#[cfg(feature = "all-fractals")]
fn mandelbrot_family_pre_modify_point_inner_big(
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
    #[inline]
    #[cfg(feature = "all-fractals")]
    fn pre_modify_point(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables) {
        mandelbrot_family_pre_modify_point(consts, vars);
    }

    #[inline]
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
    #[inline]
    #[cfg(feature = "all-fractals")]
    fn pre_modify_point(consts: &RunningConstants<'a, E>, vars: &mut RunningVariables) {
        mandelbrot_family_pre_modify_point(consts, vars);
    }

    #[inline]
    fn iterate_algorithm(
        consts: &RunningConstants<'a, E>,
        vars: &mut RunningVariables,
        iters: u32,
    ) {
        mandelbrot_perturbed_iterate_algorithm(consts, vars, iters);
    }
}

#[doc(hidden)]
#[inline]
pub fn mandelbrot_perturbed_iterate_algorithm<E: Exponentiator>(
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
    if vars.ref_iter == consts.n_reference {
        vars.ref_iter = 0;
    }
    let z = Complex::from(consts.reference_points[vars.ref_iter]) + dz_p;
    vars.norm_sqr = z.abs_sq();
    if vars.norm_sqr < dz_p.abs_sq() || vars.ref_iter == 0 {
        dz_p = z;
        vars.ref_iter = 0;
    }
    vars.z = z;
    vars.dz_perturb = dz_p;
}

/// Updates a vector of reference points.
///
/// The vector will be cleared and rewritten.
#[cfg(not(spirv))]
pub fn mandelbrot_perturbed_compute_reference_iters(
    points: &mut Vec<Vec2>,
    centre: &crate::BigVec2,
    algorithm: Algorithm,
    max_iter: u32,
) {
    use dashu_float::{FBig, round::mode as RoundingMode};

    use crate::BigComplex;

    points.clear();
    #[cfg(feature = "all-fractals")]
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
        #[cfg(feature = "all-fractals")]
        mandelbrot_family_pre_modify_point_inner_big(&mut z, modifiers);

        // <<< This is iterate_algorithm
        // TODO: Implement powers other than 2 (requires support in BigComplex)
        z = z.square();

        #[cfg(feature = "all-fractals")]
        {
            use dashu::base::Sign;
            // Algorithm difference here:
            // Celtic uses z_re_abs instead of z_re.
            // Variant may or may not take z_re_abs depending on the iters count.
            let is_odd = !iter.is_multiple_of(2);
            let use_z_re_abs = modifiers.iter_re_abs || (modifiers.iter_re_variant && is_odd);
            if use_z_re_abs && z.x.sign() == Sign::Negative {
                z.x *= Sign::Negative;
            }
        }
        z = z + &c;
        // >>> End of iterate_algorithm analogue

        iter += 1;
        points.push(z.as_vec2());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use const_default::ConstDefault as _;
    use pretty_assertions::assert_eq;

    use super::Flags;
    use crate::{
        BigVec2, Vec2,
        data::{Algorithm, BoundaryClass, FragmentConstants, Palette, PushExponent},
        engine,
        util::Size,
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
        }
    }

    #[test]
    fn mandelbrot_known_answer() {
        #![allow(clippy::float_cmp)]
        let point = crate::vec2(-0.75, 0.75);
        eprintln!("{:#?}", test_frag_consts());
        let result = engine::render(
            &test_frag_consts(),
            point - test_frag_consts().viewport_translate,
            &[Vec2::ZERO; 0],
        );
        eprintln!("{result:?}");
        assert_eq!(result.iters_fraction(), 0.522_014_14);
    }

    #[test]
    fn variant_correctness() {
        let point = crate::vec2(-0.75, 0.75);
        let mut consts = test_frag_consts();
        consts.algorithm = Algorithm::Variant;
        // Variant has a debug_assert! consistency check
        let result = engine::render(&consts, point - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("{result:?}");
    }

    #[test]
    fn distance_estimator_boundary_classification() {
        let centre = vec2(-1.5, 0.0);
        let centre_big = BigVec2::try_new(centre.x, centre.y).unwrap();
        let mut consts = FragmentConstants {
            flags: Flags::PERTURBATION_MODE | Flags::DISTANCE_ESTIMATE,
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
        assert!(run_case(17));
        assert!(run_case(18));
        assert!(run_case(19));
        assert!(run_case(20));
        assert!(run_case(34));
        // f32 has been known to experience things going a bit weird around around here, with
        // previous implementations.
        assert!(run_case(35));
        assert!(run_case(36));
        assert!(run_case(37));
        assert!(run_case(38));
        assert!(run_case(100));
    }
}
