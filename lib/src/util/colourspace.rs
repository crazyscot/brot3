//! GPU-friendly colour space representations and conversions
//!
//! (c) 2025-6 Ross Younger

use float_eq::float_eq;

#[cfg(spirv)]
use crate::Real;
use crate::Vec3;

/// RGB colour space.
///
/// Each component is in the range (0.0, 1.0).
#[derive(Clone, Copy, Debug, PartialEq, derive_more::Display)]
#[display("[r={} g={} b={}]", _0[0], _0[1], _0[2])]
pub struct RgbVec(pub Vec3);

#[allow(missing_docs)]
impl RgbVec {
    pub const BLACK: RgbVec = RgbVec(Vec3::splat(0.0));
    pub const WHITE: RgbVec = RgbVec(Vec3::splat(1.0));
}

impl From<[f32; 3]> for RgbVec {
    fn from(value: [f32; 3]) -> Self {
        Self(Vec3::from(value))
    }
}
impl From<Vec3> for RgbVec {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, derive_more::Constructor)]
/// HSL colour space
pub struct Hsl {
    /// Hue in degrees (range 0..360)
    pub h: f32,
    /// Saturation (range 0..100)
    pub s: f32,
    /// Lightness (range 0..100; 0=black, 50=fully saturated, 100=white)
    pub l: f32,
}
#[allow(missing_docs)]
impl Hsl {
    pub const BLACK: Self = Self {
        h: 0.,
        s: 0.,
        l: 0.,
    };
    pub const WHITE: Self = Self {
        h: 0.,
        s: 0.,
        l: 100.,
    };
}
impl PartialEq for Hsl {
    fn eq(&self, other: &Self) -> bool {
        float_eq!(self.h, other.h, abs <= 0.000_04)
            && float_eq!(self.s, other.s, abs <= 0.000_04)
            && float_eq!(self.l, other.l, abs <= 0.000_04)
    }
}

impl From<Hsl> for RgbVec {
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
        Self(Vec3 { x, y, z })
    }
}

impl From<RgbVec> for Hsl {
    fn from(value: RgbVec) -> Self {
        #![allow(clippy::many_single_char_names)]
        let RgbVec(Vec3 { x: r, y: g, z: b }) = value;
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let l = (max + min) * 0.5;

        let is_gray = float_eq!(max, min, abs <= 0.000_001);

        // saturation (taking care not to divide by zero)
        let d = max - min;
        let s_denom = if l > 0.5 { 2.0 - max - min } else { max + min };
        let s_general = (d / s_denom).max(0.0); // if we just divided by zero, max gets rid of the NaN
        let s = if is_gray { 0.0 } else { s_general * 100.0 };

        // compute hue components safely: avoid dividing by zero by substituting 1.0 when chroma ==
        // 0
        let safe_d = if is_gray { 1.0 } else { d };
        let t = if g < b { 6.0 } else { 0.0 };
        let h_r = ((g - b) / safe_d + t) * 60.0;
        let h_g = ((b - r) / safe_d + 2.0) * 60.0;
        let h_b = ((r - g) / safe_d + 4.0) * 60.0;

        // priority-preserving selection for which channel is max (r wins if tied)
        let mr = if float_eq!(max, r, abs <= 0.000_001) {
            1.0
        } else {
            0.0
        };
        let mg = if float_eq!(max, g, abs <= 0.000_001) {
            1.0 - mr
        } else {
            0.0
        };
        let mb = 1.0 - mr - mg;

        let h_general = h_r * mr + h_g * mg + h_b * mb;
        let h = if is_gray { 0.0 } else { h_general };

        Self { h, s, l: l * 100.0 }
    }
}

/// Packed RGBA colour as a single u32, with 8 bits per channel.
/// This is assembled as a u32, but endian swapped as necessary so that it can be directly cast to a
/// `[u8]` and written to a PNG file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackedRgba8(pub u32);

impl From<RgbVec> for PackedRgba8 {
    fn from(rgbvec: RgbVec) -> Self {
        let rgb = (rgbvec.0.clamp(Vec3::ZERO, Vec3::ONE) * 255.0)
            .round()
            .as_uvec3();
        let (r, g, b) = (rgb.x, rgb.y, rgb.z);
        if cfg!(target_endian = "big") {
            Self((r << 24) | (g << 16) | (b << 8) | 255)
        } else {
            Self((255 << 24) | (b << 16) | (g << 8) | r)
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Hsl, RgbVec};

    fn hsl_rgb_case(hsl: Hsl) {
        let rgb: RgbVec = hsl.into();
        assert!(rgb.0.x >= 0.0 && rgb.0.x <= 1.0);
        assert!(rgb.0.y >= 0.0 && rgb.0.y <= 1.0);
        assert!(rgb.0.z >= 0.0 && rgb.0.z <= 1.0);
        let hsl2: Hsl = rgb.into();
        assert_eq!(hsl, hsl2, "failing case: {hsl:?}");
    }
    #[test]
    fn hsl_rgb_tests() {
        hsl_rgb_case(Hsl::new(0., 100., 50.));
        hsl_rgb_case(Hsl::new(60., 100., 50.));
        hsl_rgb_case(Hsl::new(120., 100., 50.));
        hsl_rgb_case(Hsl::new(180., 100., 50.));
        hsl_rgb_case(Hsl::new(240., 100., 50.));
        hsl_rgb_case(Hsl::new(300., 50., 25.));
        hsl_rgb_case(Hsl::new(0., 0., 0.));
        hsl_rgb_case(Hsl::new(0., 0., 50.));
        hsl_rgb_case(Hsl::new(0., 0., 100.));
    }

    #[test]
    fn known_answer_conversions() {
        macro_rules! tc {
            // syntax: input, expected, conversion type
            ($c1:expr, $c2:expr, $t:ty) => {|| {
                let result = <$t>::from($c1);
                assert_eq!(result, $c2.into(), "failing case: {}", stringify!($c1 $c2));
            }};
        }
        let cases = [tc!(Hsl::new(240.0, 100.0, 50.0), [0.0, 0.0, 1.0], RgbVec)];
        for f in cases {
            f();
        }
    }

    #[test]
    fn known_answer_rgbvec_rgba() {
        let cases = [
            ([0.0, 0.0, 0.0], 0x0000_00ff),
            ([1.0, 1.0, 1.0], 0xffff_ffff),
            ([1.0, 0.0, 0.0], 0xff00_00ff),
            ([0.0, 1.0, 0.0], 0x00ff_00ff),
            ([0.0, 0.0, 1.0], 0x0000_ffff),
            ([0.5, 0.0, 0.25], 0x8000_40ff),
        ]
        .map(|(rgb, rgba)| (RgbVec::from(rgb), rgba));
        for (rgbvec, expected) in cases {
            let rgba: super::PackedRgba8 = rgbvec.into();
            assert_eq!(u32::from_be(rgba.0), expected, "failing case: {rgbvec:?}");
        }
    }
}
