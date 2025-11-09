//! Colouring algorithms

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use core::f32::consts::TAU;
use shader_common::enums::Colourer as ColourerSelection;
use shader_util::colourspace::{Hsl, Lch, Rgb, Vec3Rgb};

use super::{vec3, ColourStyle, FragmentConstants, PointResult};

pub fn colour_data(
    data: PointResult,
    constants: &FragmentConstants,
    _pixel_spacing: f32,
) -> Vec3Rgb {
    use ColourerSelection as CS;
    let hsl = match constants.palette.colourer {
        CS::LogRainbow => log_rainbow(constants, &data),
        CS::SqrtRainbow => sqrt_rainbow(constants, &data),
        CS::WhiteFade => white_fade(constants, &data),
        CS::BlackFade => black_fade(constants, &data),
        CS::OneLoneCoder => one_lone_coder(constants, &data),
        CS::LchGradient => lch_gradient(constants, &data),
        CS::Monochrome => monochrome(constants, &data),
        _ => todo!(),
    };
    hsl.into()
}

fn point_iters(constants: &FragmentConstants, point: &PointResult) -> f32 {
    match constants.palette.style {
        ColourStyle::Continuous => point.iters() as f32 + point.iters_fraction(),
        ColourStyle::Discrete => point.iters() as f32,
        _ => 0.0,
    }
}

fn log_rainbow(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let offset = constants.palette.offset * 36.;
    let angle: f32 =
        point_iters(constants, pixel).ln() * constants.palette.gradient * 100. + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
}

fn sqrt_rainbow(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    let offset = constants.palette.offset * 36.;
    let angle: f32 =
        point_iters(constants, pixel).sqrt() * constants.palette.gradient * 100. + offset; // DEGREES
    Hsl::new(
        angle,
        constants.palette.saturation,
        constants.palette.lightness,
    )
}

/// Based on Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn white_fade(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = point_iters(constants, pixel).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::WHITE
    } else {
        Rgb::from(vec3(
            (iters * 2.0 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.5 * grad + off).cos() * 0.5 + 0.5,
            (iters * 1.0 * grad + off).cos() * 0.5 + 0.5,
        ))
        .into()
    }
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
fn black_fade(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = point_iters(constants, pixel).ln();
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    if iters < 0.0 {
        Hsl::BLACK
    } else {
        Rgb::from(vec3(
            0.5 - (iters * 1.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 2.0 * grad + off).cos() * 0.5,
            0.5 - (iters * 3.0 * grad + off).cos() * 0.5,
        ))
        .into()
    }
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
fn one_lone_coder(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    let iters = point_iters(constants, pixel);
    let grad = constants.palette.gradient;
    // Offset is applied before cos(), so scale the input (0..10) to 2pi
    let off = constants.palette.offset * TAU / 10.;
    Rgb::from(vec3(
        (0.1 * grad * iters + off).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 2.094).sin() * 0.5 + 0.5,
        (0.1 * grad * iters + off + 4.188).sin() * 0.5 + 0.5,
    ))
    .into()
}

fn monochrome(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Compute an input from 0..1, relative to max_iter
    let input = point_iters(constants, pixel).ln() / (constants.max_iter as f32).ln();
    // Scale the offset down to -2..2
    let offset = constants.palette.offset / 5.;
    // This palette has a gamma transfer function
    let shade: f32 = input.powf(constants.palette.gamma) * constants.palette.gradient + offset;
    Hsl::new(0., 0., shade * 100.0)
}

/// LCH Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring>
fn lch_gradient(constants: &FragmentConstants, pixel: &PointResult) -> Hsl {
    if pixel.inside() {
        return Hsl::BLACK;
    }
    // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
    let offset = constants.palette.offset * 36.;

    let s: f32 = point_iters(constants, pixel) / constants.max_iter as f32;
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
    use float_eq::float_eq;
    use shader_common::enums::{Algorithm, ColourStyle, Colourer};
    use shader_common::{FragmentConstants, Palette};

    fn vec3_eq(a: Vec3Rgb, b: Vec3Rgb) -> bool {
        float_eq!(a.x, b.x, abs <= 0.000_04)
            && float_eq!(a.y, b.y, abs <= 0.000_04)
            && float_eq!(a.z, b.z, abs <= 0.000_04)
    }

    #[test]
    fn hsl_known_answer() {
        let consts = FragmentConstants::default();
        let data = PointResult::new_outside(100, 0.0, 1.0, 0., 0.);
        let expected = Vec3Rgb::from([0.3247156, 1., 0.]);
        assert!(vec3_eq(expected, super::colour_data(data, &consts, 0.0)));
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
        let expected = Vec3Rgb::from([0.901042, 0.3573773, 0.]);
        let result = super::colour_data(data, &consts, 0.0);
        assert!(vec3_eq(result, expected));
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
        let expected = Vec3Rgb::from([0.47777647, 0.03193772, 0.1543931]);
        let result = super::colour_data(data, &consts, 0.0);
        assert!(vec3_eq(result, expected));
    }
}
