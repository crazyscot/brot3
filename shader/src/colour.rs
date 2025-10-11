//! Colouring algorithms

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use core::f32::consts::TAU;
use shader_common::Colourer as ColourerSelection;
use shader_util::colourspace::{Hsl, Lch, Vec3Rgb};

use super::{FragmentConstants, PointResult, f32::vec3};

pub fn colour_data(data: PointResult, constants: &FragmentConstants) -> Vec3Rgb {
    use ColourerSelection as CS;
    if data.iters == u32::MAX {
        return Vec3Rgb::ZERO;
    }
    match constants.palette.colourer {
        CS::LogRainbow => log_rainbow(constants, &data),
        CS::SqrtRainbow => sqrt_rainbow(constants, &data),
        CS::WhiteFade => white_fade(constants, &data),
        CS::BlackFade => black_fade(constants, &data),
        CS::OneLoneCoder => one_lone_coder(constants, &data),
        CS::LchGradient => lch_gradient(constants, &data),
        CS::Monochrome => monochrome(constants, &data),
    }
}

fn log_rainbow(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    let offset = constants.palette.offset * 36.;
    let angle: f32 =
        pixel.value(constants.fractional_iters.into()).ln() * constants.palette.gradient * 100.
            + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
    .into()
}

fn sqrt_rainbow(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    let offset = constants.palette.offset * 36.;
    let angle: f32 =
        pixel.value(constants.fractional_iters.into()).sqrt() * constants.palette.gradient * 100.
            + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
    .into()
}

/// Based on Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn white_fade(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    let iters = pixel.value(constants.fractional_iters.into()).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        vec3(1., 1., 1.)
    } else {
        vec3(
            (iters * 2.0 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.5 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.0 * grad + off).cos() * 0.5 + 0.5,
        )
    }
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn black_fade(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    let iters = pixel.value(constants.fractional_iters.into()).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        vec3(0., 0., 0.)
    } else {
        vec3(
            0.5 - (iters * 1.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 2.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 3.0 * grad + off).cos() * 0.5,
        )
    }
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
fn one_lone_coder(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    let iters = pixel.value(constants.fractional_iters.into());
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    vec3(
        (0.1 * grad * iters + off).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 2.094).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 4.188).sin() * 0.5 + 0.5,
    )
}

fn monochrome(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    // Compute an input from 0..1, relative to max_iter
    let input =
        pixel.value(constants.fractional_iters.into()).ln() / (constants.max_iter as f32).ln();
    // Scale the offset down to -2..2
    let offset = constants.palette.offset / 5.;
    // This palette has a gamma transfer function
    let shade: f32 = input.powf(constants.palette.gamma) * constants.palette.gradient + offset;
    Vec3Rgb::splat(shade)
}

/// LCH Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring>
fn lch_gradient(constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
    if pixel.iters == u32::MAX {
        return vec3(0., 0., 0.);
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    let offset = constants.palette.offset * 36.;

    let s: f32 = pixel.value(constants.fractional_iters.into()) / constants.max_iter as f32;
    let v1 = (core::f32::consts::PI * s).cos();
    let v2 = 1.0 - v1 * v1;
    let lightness = 75.0 - (75.0 * v2);
    let hue = (s * 360.0 * constants.palette.gradient).powf(1.5) + offset;
    let lch = Lch::new(lightness, 28.0 + lightness, hue);
    lch.into()
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{PointResult, Vec3Rgb};
    use shader_common::{Colourer, FragmentConstants, Palette};
    #[test]
    fn hsl_known_answer() {
        let consts = FragmentConstants::default();
        let data = PointResult {
            iters: 100,
            fractional_iters: 100.0,
        };
        let expected = Vec3Rgb::from([0.3247156, 1., 0.]);
        assert_eq!(expected, super::colour_data(data, &consts));
    }

    #[test]
    fn lch_known_answer() {
        let consts = FragmentConstants {
            max_iter: 100,
            palette: Palette {
                colourer: Colourer::LchGradient,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(consts.algorithm, shader_common::Algorithm::Mandelbrot);
        let data = PointResult {
            iters: 5,
            fractional_iters: 4.31876,
        };
        let expected = Vec3Rgb::from([0.901042, 0.3573773, 0.]);
        let result = super::colour_data(data, &consts);
        assert_eq!(result, expected);
    }
}
