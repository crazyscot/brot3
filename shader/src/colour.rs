//! Colouring algorithms

#![allow(missing_docs)]

#[cfg(not(target_arch = "spirv"))]
const DEBUG_COLOUR: bool = false;

macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "spirv"))]
        if DEBUG_COLOUR {
            eprintln!($($arg)*);
        }
    };
}

use core::f32::consts::TAU;

use shader_common::enums::Modifier;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{
    FragmentConstants, PointResult,
    colourspace::{Hsl, Lch, RgbVec},
};

#[must_use]
pub fn colour_data(data: PointResult, constants: &FragmentConstants, pixel_spacing: f32) -> RgbVec {
    use shader_common::enums::Colourer as C;
    let iters = data.iters(constants.palette.colour_style);
    let mut hsl = match constants.palette.colourer {
        C::LogRainbow => log_rainbow(constants, iters, &data),
        C::SqrtRainbow => sqrt_rainbow(constants, iters, &data),
        C::WhiteFade => white_fade(constants, iters, &data),
        C::BlackFade => black_fade(constants, iters, &data),
        C::OneLoneCoder => one_lone_coder(constants, iters, &data),
        C::LchGradient => lch_gradient(constants, iters, &data),
        C::Monochrome => monochrome(constants, iters, &data),
        C::None => Hsl::WHITE,
        _ => Hsl::BLACK,
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

fn factor_for(input: f32, style: Modifier, pixel_spacing: f32, data: &PointResult) -> f32 {
    let factor = match style {
        shader_common::enums::Modifier::Filaments1 => {
            if data.inside() {
                return 100.0;
            }
            dist_value(data.distance(), pixel_spacing) /* 0..1 */
        }
        shader_common::enums::Modifier::Filaments2 => {
            if data.inside() {
                return 0.0;
            }
            dist_value(data.distance(), pixel_spacing) /* 0..1 */
        }
        shader_common::enums::Modifier::FinalAngle => data.angle() / TAU + 0.5,
        shader_common::enums::Modifier::FinalRadius => {
            let factor = data.radius_sqr() / crate::fractal::ESCAPE_THRESHOLD_SQ;
            deprintln!("rsqr {}, factor {factor}", data.radius_sqr());
            factor
        }
        _ => 1.0,
    };
    factor * input
}

fn dist_value(distance: f32, pixel_spacing: f32) -> f32 {
    let dscale = (distance / pixel_spacing).log2();
    if dscale > 0.0 {
        1.0
    } else if dscale > -8.0 {
        (8.0 + dscale) / 8.0
    } else {
        0.0
    }
}

fn log_rainbow(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 ===
    // 360.
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let offset = constants.palette.offset * 36.;
    let angle: f32 = iters.ln() * constants.palette.gradient * 100. + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
}

fn sqrt_rainbow(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 ===
    // 360.
    let offset = constants.palette.offset * 36.;
    let angle: f32 = iters.sqrt() * constants.palette.gradient * 100. + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
}

/// Based on Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn white_fade(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = iters.ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::WHITE
    } else {
        RgbVec::from([
            (iters * 2.0 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.5 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.0 * grad + off).cos() * 0.5 + 0.5,
        ])
        .into()
    }
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn black_fade(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = iters.ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::BLACK
    } else {
        RgbVec::from([
            0.5 - (iters * 1.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 2.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 3.0 * grad + off).cos() * 0.5,
        ])
        .into()
    }
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
fn one_lone_coder(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    RgbVec::from([
        (0.1 * grad * iters + off).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 2.094).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 4.188).sin() * 0.5 + 0.5,
    ])
    .into()
}

#[allow(clippy::cast_precision_loss)]
fn monochrome(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Compute an input from 0..1, relative to max_iter
    let input = iters.ln() / (constants.max_iter as f32).ln();
    // Scale the offset down to -2..2
    let offset = constants.palette.offset / 5.;
    // This palette has a gamma transfer function
    let shade: f32 = input.powf(constants.palette.gamma) * constants.palette.gradient + offset;
    Hsl::new(0., 0., shade * 100.0)
}

/// LCH Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring>
#[allow(clippy::cast_precision_loss)]
fn lch_gradient(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 ===
    // 360.
    let offset = constants.palette.offset * 36.;

    let s: f32 = iters / constants.max_iter as f32;
    let v1 = (core::f32::consts::PI * s).cos();
    let lightness = 75.0 * v1 * v1;
    let hue = (s * 360.0 * constants.palette.gradient).powf(1.5) + offset;
    let lch = Lch::new(lightness + 25.0, lightness + 35.0, hue);
    lch.into()
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use float_eq::float_eq;
    use shader_common::{
        FragmentConstants, Palette,
        enums::{Algorithm, ColourStyle, Colourer, Modifier},
    };

    use super::{PointResult, RgbVec};
    use crate::Vec3;

    macro_rules! assert_rgbvec_eq {
        ($a:expr, $b:expr) => {
            assert!(
                float_eq!($a.0.x, $b.0.x, abs <= 0.000_04)
                    && float_eq!($a.0.y, $b.0.y, abs <= 0.000_04)
                    && float_eq!($a.0.z, $b.0.z, abs <= 0.000_04),
                "float mismatch: {:?} != {:?}",
                $a,
                $b
            );
        };
    }

    #[test]
    fn hsl_known_answer() {
        let consts = FragmentConstants::default();
        let data = PointResult::new_outside(100, 0.0, 1.0, 0., 0.);
        let expected = RgbVec::from([0.324_715_6, 1., 0.]);
        assert_rgbvec_eq!(expected, super::colour_data(data, &consts, 0.0));
    }

    #[test]
    fn lch_known_answer() {
        let consts = FragmentConstants {
            max_iter: 100,
            palette: Palette::default()
                .with_colourer(Colourer::LchGradient)
                .with_style(ColourStyle::Discrete),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);
        let data = PointResult::new_outside(5, 0.31876, 1.0, 0., 0.);
        let expected = RgbVec::from([1., 0.782_427_3, 0.]);
        let result = super::colour_data(data, &consts, 0.0);
        assert_rgbvec_eq!(result, expected);
    }

    #[test]
    fn white_fade_known_answer() {
        let consts = FragmentConstants {
            max_iter: 100,
            palette: Palette::default().with_colourer(Colourer::WhiteFade),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);
        let data = PointResult::new_outside(10, 0.31876, 1.0, 0., 0.);
        let expected = RgbVec::from([0.477_776_47, 0.031_937_72, 0.154_393_1]);
        let result = super::colour_data(data, &consts, 0.0);
        assert_rgbvec_eq!(result, expected);
    }

    #[test]
    fn filaments() {
        use spirv_std::glam::{uvec2, vec2};
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments1),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        consts.viewport_zoom = 0.83;
        let pixel_size =
            FragmentConstants::pixel_spacing_f32(consts.size.height, consts.viewport_zoom);
        let pt = vec2(-0.707_752, -0.353_065_3);
        consts.viewport_translate = pt;
        let data = crate::fractal::render(&consts, pt);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec::BLACK);
    }

    #[test]
    fn filaments2() {
        use spirv_std::glam::{Vec2, uvec2};
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments1),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        // Origin (0,0), zoom 30 => the viewport is filled by the cardioid
        consts.viewport_zoom = 30.0;
        let pixel_size =
            FragmentConstants::pixel_spacing_f32(consts.size.height, consts.viewport_zoom);
        let pt = Vec2::splat(0.1);
        consts.viewport_translate = pt;
        let data = crate::fractal::render(&consts, pt);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec::WHITE);
    }

    #[test]
    fn filaments3() {
        use spirv_std::glam::{uvec2, vec2};
        let mut consts = FragmentConstants {
            max_iter: 200,
            palette: Palette::default()
                .with_colourer(Colourer::None)
                .with_brightness(Modifier::Filaments1),
            size: uvec2(500, 500).into(),
            ..Default::default()
        };
        assert_eq!(consts.algorithm, Algorithm::Mandelbrot);

        consts.viewport_zoom = 4.0;
        let pixel_size =
            FragmentConstants::pixel_spacing_f32(consts.size.height, consts.viewport_zoom);
        let pt = vec2(-0.8789, -0.23563);
        consts.viewport_translate = pt;
        let data = crate::fractal::render(&consts, pt);
        eprintln!("data: {data:?}");
        data.assert_no_subnormals();
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, RgbVec::BLACK);
    }

    #[test]
    fn radius() {
        use spirv_std::glam::{uvec2, vec2};
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
        let pixel_size =
            FragmentConstants::pixel_spacing_f32(consts.size.height, consts.viewport_zoom);
        let pt = vec2(0.17388, 0.80085);
        consts.viewport_translate = pt;
        let data = crate::fractal::render(&consts, pt);
        eprintln!("data: {data:?}");
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, Vec3::splat(0.325_493_5).into());
    }
}
