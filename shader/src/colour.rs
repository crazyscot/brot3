//! Colouring algorithms

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use core::f32::consts::TAU;
use shader_common::Colourer as ColourerSelection;
use shader_util::colourspace::{Hsl, Vec3Rgb};

use super::{FragmentConstants, PointResult, f32::vec3};

pub fn colour_data(data: PointResult, constants: &FragmentConstants) -> Vec3Rgb {
    use ColourerSelection as CS;
    if data.iters == u32::MAX {
        return Vec3Rgb::ZERO;
    }
    match constants.palette.colourer {
        CS::LogRainbow => LogRainbow {}.colour(constants, &data),
        CS::SqrtRainbow => SqrtRainbow {}.colour(constants, &data),
        CS::WhiteFade => WhiteFade {}.colour(constants, &data),
        CS::BlackFade => BlackFade {}.colour(constants, &data),
        CS::OneLoneCoder => OneLoneCoder {}.colour(constants, &data),
    }
}

trait RgbColourer {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb;
}

struct LogRainbow {}
impl RgbColourer for LogRainbow {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
        let offset = constants.palette.offset * 36.;
        let angle: f32 = pixel.smooth_iters.ln() * constants.palette.gradient * 100. + offset; // DEGREES
        Hsl::new(
            angle,
            constants.palette.saturation as f32,
            constants.palette.lightness as f32,
        )
        .into()
    }
}

struct SqrtRainbow {}
impl RgbColourer for SqrtRainbow {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        // Input offset range is 0..10. As we're operating with a hue angle, scale it so that 0.0 === 360.
        let offset = constants.palette.offset * 36.;
        let angle: f32 = pixel.smooth_iters.sqrt() * constants.palette.gradient * 100. + offset; // DEGREES
        Hsl::new(
            angle,
            constants.palette.saturation as f32,
            constants.palette.lightness as f32,
        )
        .into()
    }
}

/// Based on Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
struct WhiteFade {}
impl RgbColourer for WhiteFade {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters.ln();
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
}

/// Based on Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
struct BlackFade {}
impl RgbColourer for BlackFade {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters.ln();
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
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
struct OneLoneCoder {}
impl RgbColourer for OneLoneCoder {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters;
        let grad = constants.palette.gradient;
        // Offset is applied before cos(), so scale the input (0..10) to 2pi
        let off = constants.palette.offset * TAU / 10.;
        vec3(
            (0.1 * grad * iters + off).sin() * 0.5 + 0.5,
            (0.1 * grad * iters + off + 2.094).sin() * 0.5 + 0.5,
            (0.1 * grad * iters + off + 4.188).sin() * 0.5 + 0.5,
        )
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{PointResult, Vec3Rgb};
    use shader_common::FragmentConstants;
    #[test]
    fn hsl_known_answer() {
        let consts = FragmentConstants::default();
        let data = PointResult {
            iters: 100,
            smooth_iters: 100.0,
        };
        let expected = Vec3Rgb::from([0.3247156, 1., 0.]);
        assert_eq!(expected, super::colour_data(data, &consts));
    }
}
