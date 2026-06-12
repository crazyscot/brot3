//! Colouring algorithms
//!
//! These implementations are (c) 2025-6 Ross Younger, with original authors as noted.

#![allow(missing_docs)]

#[cfg(not(spirv))]
const DEBUG_COLOUR: bool = false;

#[clippy::format_args]
macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(spirv))]
        if DEBUG_COLOUR {
            eprintln!($($arg)*);
        }
    };
}

use core::f32::consts::{FRAC_PI_2, PI, TAU};

#[cfg(spirv)]
use crate::Real;
use crate::{
    Vec3,
    data::{BoundaryClass, Colourer, FragmentConstants, Modifier, PointResult},
    util::RgbVec,
    vec3,
};
const ONE: Vec3 = Vec3::ONE;
const ZERO: Vec3 = Vec3::ZERO;

#[must_use]
/// Computes the colour of a point based on the provided colouring algorithm and parameters.
pub fn colour_data(data: PointResult, constants: &FragmentConstants) -> RgbVec {
    let iters = Iterations(data.iters(constants.palette.colour_style));
    let mut rgb: RgbVec = match constants.palette.colourer {
        Colourer::WhiteFade
        | Colourer::BlackFade
        | Colourer::Mandy
        | Colourer::OneLoneCoder
        | Colourer::Monochrome2 => {
            colour_simple_family(constants, iters, &data, constants.palette.colourer)
        }
        Colourer::Neon2 | Colourer::IcyBlue => {
            colour_powered_family(constants, iters, &data, constants.palette.colourer)
        }
        Colourer::None => RgbVec::WHITE,
    };
    deprintln!("interim rgb: {rgb:?}");

    rgb.0 *= factor_for(constants.palette.brightness_style, &data);
    rgb
}

struct Iterations(f32);
impl Iterations {
    #[inline]
    fn add_with_floor(self, add: f32, floor: f32) -> Self {
        let tmp = self.0 + add;
        let tmp = if tmp < floor { floor } else { tmp };
        Self(tmp)
    }

    #[inline]
    fn ln(self) -> Self {
        Self(self.0.ln())
    }

    #[inline]
    fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        Self(self.0 * factor)
    }

    #[inline]
    fn add_div(self, add: f32, divisor: f32) -> Self {
        Self((self.0 + add) / divisor)
    }
}

#[inline]
fn colour_simple_family(
    constants: &FragmentConstants,
    iters: Iterations,
    pixel: &PointResult,
    colourer: Colourer,
) -> RgbVec {
    match colourer {
        Colourer::WhiteFade => simple_cos(
            constants,
            &iters.add_with_floor(-3.0, 1.0).ln(),
            pixel,
            Vec3::new(2.0, 1.5, 1.0),
            ZERO,
            0.5,
        ),
        Colourer::BlackFade => simple_cos(
            constants,
            &iters.add_with_floor(-3.0, 1.0).ln(),
            pixel,
            Vec3::new(1.0, 2.0, 3.0),
            ZERO,
            -0.5,
        ),
        Colourer::Mandy => simple_cos(
            constants,
            &iters.add_with_floor(-3.0, 0.0).sqrt().scale(TAU),
            pixel,
            vec3(
                0.2,       /* 1/5 */
                0.14285,   /* 1/7 */
                0.090_090, /* 1/11 */
            ),
            ZERO,
            0.5,
        ),
        Colourer::OneLoneCoder => simple_cos(
            constants,
            &iters.scale(0.1),
            pixel,
            ONE,
            vec3(
                -FRAC_PI_2,
                -FRAC_PI_2 + TAU / 3.0,
                -FRAC_PI_2 + 2.0 * TAU / 3.0,
            ),
            0.5,
        ),
        Colourer::Monochrome2 => simple_cos(
            constants,
            &iters.add_with_floor(-3.0, 1.0).ln().scale(2.0),
            pixel,
            ONE,
            ZERO,
            0.5,
        ),
        _ => RgbVec::WHITE,
    }
}

#[inline]
fn colour_powered_family(
    constants: &FragmentConstants,
    iters: Iterations,
    pixel: &PointResult,
    colourer: Colourer,
) -> RgbVec {
    match colourer {
        Colourer::Neon2 => powered_cos(
            constants,
            &iters.add_with_floor(-3.0, 1.0).ln().scale(2.0),
            pixel,
            vec3(0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0),
            Vec3::splat(-1.0),
            ONE,
            1.5,
            1.2,
        ),
        Colourer::IcyBlue => powered_cos(
            constants,
            &iters.add_div(PI, PI).ln(),
            pixel,
            vec3(PI, PI, 0.0),
            vec3(1.0, 1.0, -1.0),
            vec3(0.0, 0.0, 1.0),
            4.0,
            1.9,
        ),
        _ => RgbVec::WHITE,
    }
}

#[inline]
fn finish_colour(pixel: &PointResult, rgb: Vec3) -> RgbVec {
    if pixel.inside() {
        RgbVec::BLACK
    } else {
        RgbVec(rgb)
    }
}

#[inline]
fn scaled_palette_offset(constants: &FragmentConstants) -> f32 {
    // Offset is applied before cos(), so scale the input (0..10) to 2pi.
    constants.palette.offset * TAU / 10.0
}

#[inline]
fn simple_cos_input(
    constants: &FragmentConstants,
    iters: &Iterations,
    vec_scale: Vec3,
    vec_offset: Vec3,
) -> Vec3 {
    vec_scale * iters.0 * constants.palette.gradient + scaled_palette_offset(constants) + vec_offset
}

#[inline]
fn powered_cos_input(
    constants: &FragmentConstants,
    iters: &Iterations,
    vec_offset: Vec3,
    gamma: f32,
) -> Vec3 {
    Vec3::splat(iters.0 * constants.palette.gradient).powf(gamma)
        + scaled_palette_offset(constants)
        + vec_offset
}

fn factor_for(style: Modifier, data: &PointResult) -> f32 {
    match style {
        Modifier::Filaments => {
            if data.boundary == BoundaryClass::NotClose {
                1.0
            } else {
                0.1
            }
        }
        Modifier::FinalAngle => data.angle() / TAU + 0.5,
        Modifier::FinalRadius => {
            let factor = data.radius_sqr() / crate::ESCAPE_THRESHOLD_SQ;
            deprintln!("rsqr {}, factor {factor}", data.radius_sqr());
            factor
        }
        Modifier::Standard => 1.0,
    }
}

/// Parameterised cosine-based colouring algorithm
fn simple_cos(
    constants: &FragmentConstants,
    iters: &Iterations,
    pixel: &PointResult,
    vec_scale: Vec3,
    vec_offset: Vec3,
    cos_factor: f32,
) -> RgbVec {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return RgbVec::BLACK;
    }

    let mut v = simple_cos_input(constants, iters, vec_scale, vec_offset);
    v = v.cos() * cos_factor + Vec3::splat(0.5);

    finish_colour(pixel, v)
}

#[allow(clippy::too_many_arguments)]
/// Parameterised cosine-based colouring algorithm with power function for sharper peaks
fn powered_cos(
    constants: &FragmentConstants,
    iters: &Iterations,
    pixel: &PointResult,
    vec_offset: Vec3,
    pow_factor: Vec3,
    pow_offset: Vec3,
    power: f32,
    gamma: f32,
) -> RgbVec {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return RgbVec::BLACK;
    }

    let mut v = powered_cos_input(constants, iters, vec_offset, gamma);
    v = v.cos() * 0.5 + 0.5;
    v = v.powf(power);
    v = v * pow_factor + pow_offset;

    finish_colour(pixel, v)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(unused_imports)]
mod tests {
    use const_default::ConstDefault;
    use float_eq::float_eq;
    use strum::IntoEnumIterator;

    use super::{PointResult, RgbVec};
    use crate::{
        Vec2, Vec3,
        data::{Algorithm, Colourer, FragmentConstants, Modifier, Palette},
        engine::PixelSpacing as _,
        uvec2, vec2, vec3,
    };

    macro_rules! assert_rgbvec_near {
        ($a:expr, $b:expr) => {
            // 3 d.p. precision is enough for RGB triplets
            assert!(
                float_eq!($a.0.x, $b.0.x, abs <= 0.000_4)
                    && float_eq!($a.0.y, $b.0.y, abs <= 0.000_4)
                    && float_eq!($a.0.z, $b.0.z, abs <= 0.000_4),
                "float mismatch: {:?} != {:?}",
                $a,
                $b
            );
        };
    }

    fn is_simple_family(colourer: Colourer) -> bool {
        matches!(
            colourer,
            Colourer::WhiteFade
                | Colourer::BlackFade
                | Colourer::Mandy
                | Colourer::OneLoneCoder
                | Colourer::Monochrome2
        )
    }

    fn is_powered_family(colourer: Colourer) -> bool {
        matches!(colourer, Colourer::Neon2 | Colourer::IcyBlue)
    }

    #[test]
    fn known_answers() {
        let cases = [
            (Colourer::WhiteFade, 0, 0.1, [1.0, 1.0, 1.0]),
            (Colourer::BlackFade, 100, 0.0, [0.569, 0.981, 0.299]),
            (Colourer::BlackFade, 0, 0.1, [0.0, 0.0, 0.0]),
            (Colourer::OneLoneCoder, 100, 0.0, [0.228, 0.2725, 0.999]),
            (Colourer::Monochrome2, 100, 0.0, [0.01883, 0.01883, 0.01883]),
            (Colourer::Mandy, 100, 0.0, [0.991, 0.083, 0.8797]),
            (Colourer::Neon2, 100, 0.0, [0.702, 0.97, 0.063]),
            (Colourer::IcyBlue, 100, 0.0, [0.146, 0.146, 0.979]),
        ];
        for (colourer, iters, iters_fraction, expected) in cases {
            let consts = FragmentConstants {
                max_iter: 100_000,
                palette: Palette::default().with_colourer(colourer),
                ..Default::default()
            };
            let data = PointResult::new(
                iters,
                iters_fraction,
                0.,
                0.,
                crate::data::BoundaryClass::Indeterminate,
            );
            let expected = RgbVec::from(expected);
            let result = super::colour_data(data, &consts);
            println!("{colourer}: expected={expected} output={result}");
            assert_rgbvec_near!(result, expected);
        }
    }

    #[test]
    fn inside_pixels() {
        for c in Colourer::iter() {
            if c == Colourer::None {
                continue;
            }
            let consts = FragmentConstants {
                palette: Palette::default().with_colourer(c),
                ..Default::default()
            };
            let data = PointResult::DEFAULT;
            let result = super::colour_data(data, &consts);
            assert_eq!(result, RgbVec::BLACK, "case {c}");
        }
    }

    #[test]
    fn family_partition_covers_all_supported_colourers() {
        for colourer in Colourer::iter() {
            assert_eq!(
                is_simple_family(colourer) || is_powered_family(colourer),
                colourer != Colourer::None,
                "unexpected family coverage for {colourer}"
            );
        }
    }

    #[test]
    fn family_partition_captures_the_two_cosine_families() {
        for colourer in [
            Colourer::WhiteFade,
            Colourer::BlackFade,
            Colourer::Mandy,
            Colourer::OneLoneCoder,
            Colourer::Monochrome2,
        ] {
            assert!(
                is_simple_family(colourer),
                "expected simple-cos family for {colourer:?}"
            );
        }
        for colourer in [Colourer::Neon2, Colourer::IcyBlue] {
            assert!(
                is_powered_family(colourer),
                "expected powered-cos family for {colourer:?}"
            );
        }
    }

    #[test]
    fn all_non_none_colourers_use_shared_families() {
        for colourer in Colourer::iter() {
            if colourer == Colourer::None {
                continue;
            }
            assert!(
                is_simple_family(colourer) || is_powered_family(colourer),
                "expected recipe-family-backed implementation for {colourer}"
            );
        }
    }

    #[test]
    fn filaments() {
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        consts.viewport_zoom = 0.83;
        let pt = vec2(-0.707_752, -0.353_065_3);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.1)),);
    }

    #[test]
    fn filaments2() {
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        // Origin (0,0), zoom 30 => the viewport is filled by the cardioid
        consts.viewport_zoom = 30.0;
        let pt = Vec2::splat(0.1);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.1)));
    }

    #[test]
    fn filaments3() {
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        consts.viewport_zoom = 4.0;
        let pt = vec2(-0.8789, -0.23563);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.1)));
    }

    #[test]
    fn radius() {
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::FinalRadius),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        consts.viewport_zoom = 1.29;
        let pt = vec2(0.17388, 0.80085);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        let result = super::colour_data(data, &consts);
        eprintln!("result: {result:?}");
        assert_eq!(result, Vec3::splat(0.325_493_5).into());
    }
}
