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

use core::f32::consts::{E, PI, TAU};

use easy_cast::Conv as _;

#[cfg(spirv)]
use crate::Real;
use crate::{
    Vec3,
    data::{BoundaryClass, FragmentConstants, Modifier, PointResult},
    util::{Hsl, RgbVec},
    vec3,
};

#[must_use]
/// Computes the colour of a point based on the provided colouring algorithm and parameters.
pub fn colour_data(data: PointResult, constants: &FragmentConstants, pixel_spacing: f32) -> RgbVec {
    use crate::data::Colourer as C;
    let iters = data.iters(constants.palette.colour_style);
    let mut hsl = if cfg!(feature = "all-colourers") {
        match constants.palette.colourer {
            C::LogRainbow => log_rainbow(constants, iters, &data),
            C::WhiteFade => white_fade(constants, iters, &data),
            C::BlackFade => black_fade(constants, iters, &data),
            C::Mandy => mandy(constants, iters, &data),
            C::OneLoneCoder => one_lone_coder(constants, iters, &data),
            C::Monochrome => monochrome(constants, iters, &data),
            C::Monochrome2 => monochrome2(constants, iters, &data),
            C::Neon => neon(constants, iters, &data),
            C::IcyBlue => icyblue(constants, iters, &data),
            C::None => Hsl::WHITE,
            // _ => Hsl::BLACK,
        }
    } else {
        neon(constants, iters, &data)
    };
    deprintln!("interim hsl: {hsl:?}");

    hsl.l = factor_for(
        hsl.l,
        constants.palette.brightness_style,
        pixel_spacing,
        &data,
    );
    hsl.s = factor_for(
        hsl.s,
        constants.palette.saturation_style,
        pixel_spacing,
        &data,
    );
    hsl.into()
}

fn factor_for(input: f32, style: Modifier, _pixel_spacing: f32, data: &PointResult) -> f32 {
    let factor = match style {
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
    };
    factor * input
}

fn log_rainbow(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    // Why check pixel.inside() twice?
    // On GPU, the early return forces warp divergence, so we don't do that on spirv.
    // (On CPU, it's a cheap check that saves us doing the rest of the calculations, so we do it
    // first.)
    // Checking at the end should compile to an OpSelect instruction on spirv, which does not
    // cause warp divergence.
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 ===
    // 360.
    let offset = constants.palette.offset * 36.;
    let angle: f32 = iters.ln() * constants.palette.gradient * 100. + offset; // DEGREES
    let colour = Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    );
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// Based on Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn white_fade(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // We are using a different escape threshold to fanf, so scale the function to suit.
    let iters = (iters - 3.0).max(1.0).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    let mut v = Vec3::new(2.0, 1.5, 1.0) * iters * grad + off;
    v = (v.cos() + Vec3::ONE) * 0.5;
    let colour = RgbVec(v).into();
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn black_fade(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // We are using a different escape threshold to fanf, so scale the function to suit.
    let iters = (iters - 3.0).max(1.0).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    let mut v = Vec3::new(1.0, 2.0, 3.0) * iters * grad + off;
    v = (Vec3::ONE - v.cos()) * 0.5;
    let colour = RgbVec(v).into();
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// Based on Richard Kettlewell's "mandy". <http://www.greenend.org.uk/rjk/mandy/>
fn mandy(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Offset is applied before cos(), so scale that input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;

    // We are using a different escape threshold to rjk, so scale the function to suit.
    let iters = (iters - 3.0).max(0.0).sqrt() * TAU;
    let mut v = vec3(
        0.2,       /* 1/5 */
        0.14285,   /* 1/7 */
        0.090_090, /* 1/11 */
    );
    v = v * Vec3::splat(constants.palette.gradient * iters) + Vec3::splat(off);
    let v2 = vec3(v.x.cos(), v.y.cos(), v.z.cos()) + Vec3::ONE;
    let colour = RgbVec::from(v2 * Vec3::splat(0.5)).into();
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
fn one_lone_coder(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let grad = constants.palette.gradient;
    // Offset is applied before sin(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    let v1 = Vec3::splat(0.1 * grad * iters + off);
    // Phase shift the three channels 120 degrees apart to create an RGB cycle.
    let v2 = vec3(0.0, TAU / 3.0, 2.0 * TAU / 3.0);
    let v3 = (v1 + v2).sin() * 0.5 + 0.5;
    let colour = RgbVec(v3).into();
    if pixel.inside() { Hsl::BLACK } else { colour }
}

fn monochrome(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Compute an input from 0..1, relative to max_iter
    let input = iters.ln() / f32::conv(constants.max_iter).ln();
    // Scale the offset down to -2..2
    let offset = constants.palette.offset / 5.;
    // This palette has a gamma transfer function
    let shade: f32 = input.powf(constants.palette.gamma) * constants.palette.gradient + offset;
    let colour = Hsl::new(0., 0., shade * 100.0);
    if pixel.inside() { Hsl::BLACK } else { colour }
}

fn monochrome2(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = (iters - 3.0).max(1.0).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    let r = iters * grad * 2.0 + off;
    let l = f32::midpoint(r.cos(), 1.0);
    let colour = Hsl::new(0., 0., l * 100.0);
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// Based on the `neon` theme by David Bau <https://github.com/davidbau/mandelbrot/blob/main/index.html>
///
/// Creates vibrant, intense colours by cycling three sine waves with saturation boost.
/// One channel is always near zero. One channel is always near one.
fn neon(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let grad = constants.palette.gradient.sqrt();
    // Offset is applied before sin(), so scale the input (0..10) to 2pi
    let offset = constants.palette.offset * TAU / 10.;

    // Apply minimum 10 iters to reduce visual noise
    let angle = (iters + 10.0).ln() * 0.8 * PI * grad + offset;

    let mut v = (Vec3::new(0.0, PI * 0.66667, PI * 1.33333) + angle)
        .sin()
        .abs();
    // Suppress minimum channel to boost saturation
    //let min_ch = r.min(g).min(b) * 0.8;
    let min_ch = v.min_element() * 0.8;
    v = (v - min_ch).max(Vec3::ZERO);
    // Normalise
    //let max_ch = r.max(g).max(b);
    let max_ch = v.max_element();
    v /= max_ch.max(0.0);
    let colour = RgbVec(v).into();
    if pixel.inside() { Hsl::BLACK } else { colour }
}

/// A cool appearance with blue hues.
/// Inspired by the `iceblue` theme by David Bau <https://github.com/davidbau/mandelbrot/blob/main/index.html>
fn icyblue(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    #[cfg(not(spirv))]
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Log-scale input from 0..1, relative to max_iter
    let input = (iters + E).ln().ln();
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let offset = constants.palette.offset * TAU / 10.;
    // This palette has a gamma transfer function
    let r = (input * constants.palette.gradient * 2.5).powf(constants.palette.gamma) + offset;
    let shade = f32::midpoint(r.cos(), 1.0);
    let colour = Hsl::new(240., 100. * (1.0 - shade / 2.0), shade * 100.0);
    if pixel.inside() { Hsl::BLACK } else { colour }
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
        uvec2, vec2,
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

    #[test]
    fn known_answers() {
        #[cfg(feature = "all-colourers")]
        let cases = [
            (Colourer::LogRainbow, 100, 0.0, [0.325, 1., 0.]),
            (Colourer::WhiteFade, 10, 0.31876, [0.166, 0.006, 0.296]),
            (Colourer::WhiteFade, 0, 0.1, [1.0, 1.0, 1.0]),
            (Colourer::BlackFade, 100, 0.0, [0.569, 0.981, 0.299]),
            (Colourer::BlackFade, 0, 0.1, [0.0, 0.0, 0.0]),
            (Colourer::OneLoneCoder, 100, 0.0, [0.228, 0.2725, 0.999]),
            (Colourer::Monochrome, 100, 0.0, [0.175, 0.175, 0.175]),
            (Colourer::Monochrome2, 100, 0.0, [0.01883, 0.01883, 0.01883]),
            (Colourer::Neon, 100, 0.0, [0.609, 1.0, 0.078]),
            (Colourer::Mandy, 100, 0.0, [0.991, 0.083, 0.8797]),
            (Colourer::IcyBlue, 100, 0.0, [0.9717, 0.9717, 0.9908]),
        ];
        #[cfg(not(feature = "all-colourers"))]
        let cases = [(Colourer::Neon, 100, 0.0, [0.609, 1.0, 0.078])];
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
            let result = super::colour_data(data, &consts, 0.0);
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
            let result = super::colour_data(data, &consts, 0.0);
            assert_eq!(result, RgbVec::BLACK, "case {c}");
        }
    }

    #[cfg(feature = "all-colourers")]
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
        let pixel_size = consts.viewport_zoom.pixel_spacing(consts.size.height);
        let pt = vec2(-0.707_752, -0.353_065_3);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.099_999_994)));
    }

    #[cfg(feature = "all-colourers")]
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
        let pixel_size = consts.viewport_zoom.pixel_spacing(consts.size.height);
        let pt = Vec2::splat(0.1);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.099_999_994)));
    }

    #[cfg(feature = "all-colourers")]
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
        let pixel_size = consts.viewport_zoom.pixel_spacing(consts.size.height);
        let pt = vec2(-0.8789, -0.23563);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec(Vec3::splat(0.099_999_994)));
    }

    #[test]
    #[cfg(feature = "all-colourers")]
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
        let pixel_size = consts.viewport_zoom.pixel_spacing(consts.size.height);
        let pt = vec2(0.17388, 0.80085);
        let data = crate::engine::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, Vec3::splat(0.325_493_5).into());
    }
}
