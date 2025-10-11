//! Colouring algorithms

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use shader_util::colourspace::{Hsl, Vec3Rgb};

use super::{FragmentConstants, PointResult};

pub fn colour_data(data: PointResult, constants: &FragmentConstants) -> Vec3Rgb {
    if data.iters == u32::MAX {
        return Vec3Rgb::ZERO;
    }
    LogRainbow {}.colour(constants, &data)
}

trait Colourer {
    fn colour(&self, constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb;
}
// TODO there might become an HsvColourer, etc.

struct LogRainbow {}
impl Colourer for LogRainbow {
    fn colour(&self, _constants: &FragmentConstants, pixel: &PointResult) -> Vec3Rgb {
        let angle: f32 = pixel.smooth_iters.ln() * 60.0; // DEGREES
        Hsl::new(angle, 100., 50.).into()
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
