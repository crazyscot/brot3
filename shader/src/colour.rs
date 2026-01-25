//! Colouring algorithms

#![allow(missing_docs)]

#[cfg(not(target_arch = "spirv"))]
const DEBUG_COLOUR: bool = false;

#[clippy::format_args]
macro_rules! deprintln {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "spirv"))]
        if DEBUG_COLOUR {
            eprintln!($($arg)*);
        }
    };
}

use core::f32::consts::TAU;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::{
    FragmentConstants, PointResult, Vec3,
    colourspace::{Hsl, Lch, RgbVec},
    enums::Modifier,
};

/// Computes the sine of all the members of a vector
/// (syntactic sugar; glam 0.31 provides this directly)
fn vec_sin(vec: Vec3) -> Vec3 {
    Vec3::new(vec.x.sin(), vec.y.sin(), vec.z.sin())
}

/// Computes the cosine of all the members of a vector
/// (syntactic sugar; glam 0.31 provides this directly)
fn vec_cos(vec: Vec3) -> Vec3 {
    Vec3::new(vec.x.cos(), vec.y.cos(), vec.z.cos())
}

#[must_use]
pub fn colour_data(data: PointResult, constants: &FragmentConstants, pixel_spacing: f32) -> RgbVec {
    use super::enums::Colourer as C;
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
        // _ => Hsl::BLACK,
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
        Modifier::Filaments1 => {
            if data.inside() {
                return 100.0;
            }
            dist_value(data.distance(), pixel_spacing) /* 0..1 */
        }
        Modifier::Filaments2 => {
            if data.inside() {
                return 0.0;
            }
            dist_value(data.distance(), pixel_spacing) /* 0..1 */
        }
        Modifier::FinalAngle => data.angle() / TAU + 0.5,
        Modifier::FinalRadius => {
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
    // We are using a different escape threshold to fanf, so scale the function to suit.
    let iters = (iters - 3.0).max(0.0).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::WHITE
    } else {
        let mut v = Vec3::new(2.0, 1.5, 1.0) * iters * grad + off;
        v = (vec_cos(v) + Vec3::ONE) * 0.5;
        RgbVec(v).into()
        // TODO: Benchmark this on GPU, look for optimisations. Vector or not?
    }
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn black_fade(constants: &FragmentConstants, iters: f32, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // We are using a different escape threshold to fanf, so scale the function to suit.
    let iters = (iters - 3.0).max(0.0).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::BLACK
    } else {
        let mut v = Vec3::new(1.0, 2.0, 3.0) * iters * grad + off;
        v = (Vec3::ONE - vec_cos(v)) * 0.5;
        RgbVec(v).into()
        // TODO: Benchmark this on GPU, look for optimisations. Vector or not?
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
    // TODO: Benchmark this on GPU, consider vectorising.
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
#[allow(clippy::missing_panics_doc)]
mod tests {
    use const_default::ConstDefault;
    use float_eq::float_eq;
    use glam::Vec2;
    use strum::IntoEnumIterator;

    use super::{PointResult, RgbVec};
    use crate::{
        FragmentConstants, Palette, Vec3,
        enums::{Algorithm, Colourer, Modifier},
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
        let cases = [
            (Colourer::LogRainbow, 100, 0.0, [0.325, 1., 0.]),
            (Colourer::SqrtRainbow, 100, 0.0, [0.667, 0., 1.]),
            (Colourer::WhiteFade, 10, 0.31876, [0.166, 0.006, 0.296]),
            (Colourer::WhiteFade, 0, 0.1, [1.0, 1.0, 1.0]),
            (Colourer::BlackFade, 100, 0.0, [0.569, 0.981, 0.299]),
            (Colourer::BlackFade, 0, 0.1, [0.0, 0.0, 0.0]),
            (Colourer::OneLoneCoder, 100, 0.0, [0.228, 0.2725, 0.999]),
            (Colourer::LchGradient, 100, 0.0, [1.0, 0.23, 1.0]),
            (Colourer::Monochrome, 100, 0.0, [0.175, 0.175, 0.175]),
        ];
        for (colourer, iters, iters_fraction, expected) in cases {
            let consts = FragmentConstants {
                max_iter: 100_000,
                palette: Palette::default().with_colourer(colourer),
                ..Default::default()
            };
            let data = PointResult::new(iters, iters_fraction, 1.0, 0., 0.);
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
        let data =
            crate::fractal::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
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
        let data =
            crate::fractal::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
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
        let data =
            crate::fractal::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
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
        let data =
            crate::fractal::render(&consts, pt - consts.viewport_translate, &[Vec2::ZERO; 0]);
        eprintln!("data: {data:?}");
        let result = super::colour_data(data, &consts, pixel_size);
        eprintln!("result: {result:?}");
        assert_eq!(result, Vec3::splat(0.325_493_5).into());
    }
}
