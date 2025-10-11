//! Colouring algorithms

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use shader_common::Colourer as ColourerSelection;
use shader_util::colourspace::{Hsl, Vec3Rgb};

use super::{FragmentConstants, PointResult, f32::vec3};

pub fn colour_data(data: PointResult, constants: &FragmentConstants) -> Vec3Rgb {
    use ColourerSelection as CS;
    if data.iters == u32::MAX {
        return Vec3Rgb::ZERO;
    }
    match constants.colourer {
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
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let angle: f32 = pixel.smooth_iters.ln() * 60.0; // DEGREES
        Hsl::new(angle, 100., 50.).into()
    }
}

struct SqrtRainbow {}
impl RgbColourer for SqrtRainbow {
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let angle: f32 = pixel.smooth_iters.sqrt() * 20.0; // DEGREES
        Hsl::new(angle, 100., 50.).into()
    }
}

/// Tony Finch's "White Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
struct WhiteFade {}
impl RgbColourer for WhiteFade {
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters.ln();
        if iters < 0.0 {
            vec3(1., 1., 1.)
        } else {
            vec3(
                (iters * 2.0).cos() * 0.5 + 0.5,
                (iters * 1.5).cos() * 0.5 + 0.5,
                (iters * 1.0).cos() * 0.5 + 0.5,
            )
        }
    }
}

/// Tony Finch's "Black Fade" colourer
/// <https://dotat.at/prog/mandelbrot/>
struct BlackFade {}
impl RgbColourer for BlackFade {
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters.ln();
        if iters < 0.0 {
            vec3(0., 0., 0.)
        } else {
            vec3(
                0.5 - (iters * 1.0).cos() * 0.5,
                0.5 - (iters * 2.0).cos() * 0.5,
                0.5 - (iters * 3.0).cos() * 0.5,
            )
        }
    }
}

/// Colouring algorithm by `OneLoneCoder.com`
/// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
struct OneLoneCoder {}
impl RgbColourer for OneLoneCoder {
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let iters = pixel.smooth_iters;
        vec3(
            (0.1 * iters).sin() * 0.5 + 0.5,
            (0.1 * iters + 2.094).sin() * 0.5 + 0.5,
            (0.1 * iters + 4.188).sin() * 0.5 + 0.5,
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
        let expected = Vec3Rgb::from([0.60517025, 0., 1.]); // from the previous brot3's log-rainbow
        assert_eq!(expected, super::colour_data(data, &consts));
    }
}
