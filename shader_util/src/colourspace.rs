//! GPU-friendly colour space conversions

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

/// RGB pixel format data. Each component is in the range (0.0, 1.0)
pub use super::Vec3 as Vec3Rgb;

#[derive(Clone, Copy, Debug, derive_more::Constructor)]
/// HSL colour space
pub struct Hsl {
    /// Hue in degrees (range 0-360)
    h: f32,
    /// Saturation (range 0-100)
    s: f32,
    /// Lightness (range 0-100; 0=black, 50=fully saturated, 100=white)
    l: f32,
}
impl From<Hsl> for Vec3Rgb {
    fn from(value: Hsl) -> Self {
        // this algorithm is based on CSS Color 4 section 7.1 and cribbed from the color crate
        // (sadly, the color crate does not currently function in the rust-gpu environment)
        let sat = value.s * 0.01;
        let light = value.l * 0.01;
        let a = sat * light.min(1.0 - light);
        let hue_component = |n: f32| {
            let x = n + value.h * (1.0 / 30.0);
            let k = x - 12.0 * (x * (1.0 / 12.0)).floor();
            light - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
        };
        let [x, y, z] = [hue_component(0.), hue_component(8.), hue_component(4.)];
        Self { x, y, z }
    }
}
