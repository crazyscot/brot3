//! GPU-friendly colour space representations and conversions

use float_eq::float_eq;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use super::Vec3;

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

#[derive(Clone, Copy, Debug)]
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

    #[must_use]
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }
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
        #[allow(clippy::cast_precision_loss)]
        let safe_d = if is_gray { 1.0 } else { d };
        let t = if g < b { 6.0 } else { 0.0 };
        let h_r = ((g - b) / safe_d + t) * 60.0;
        let h_g = ((b - r) / safe_d + 2.0) * 60.0;
        let h_b = ((r - g) / safe_d + 4.0) * 60.0;

        // priority-preserving selection for which channel is max (r wins if tied)
        #[allow(clippy::cast_precision_loss)]
        let mr = u32::from(float_eq!(max, r, abs <= 0.000_001)) as f32;
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

#[derive(Clone, Copy, Debug)]
/// LCH colour space
pub struct Lch {
    /// Lightness (range 0..100)
    pub l: f32,
    /// Chroma (range 0..100)
    pub c: f32,
    /// Hue (degrees)
    pub h: f32,
}
impl Lch {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h }
    }
}

#[derive(Clone, Copy, Debug)]
/// CIE L*a*b* colour space
pub struct Lab {
    /// Lightness (range 0..100)
    pub l: f32,
    /// Red-green axis (range -100..100)
    pub a: f32,
    /// Yellow-blue axis (range -100..100)
    pub b: f32,
}

impl Lab {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }
}

impl From<Lch> for Lab {
    fn from(value: Lch) -> Self {
        let (sin, cos) = value.h.to_radians().sin_cos();
        let a = value.c * cos;
        let b = value.c * sin;
        Self { l: value.l, a, b }
    }
}

// Matrix from <https://docs.rs/color/0.3.2/src/color/colorspace.rs.html>: original source is CSS Color 4.
const LAB_XYZ_TO_SRGB: [[f32; 3]; 3] = [
    [3.022_233_7, -1.617_386, -0.404_847_65],
    [-0.943_848_25, 1.916_254_4, 0.027_593_868],
    [0.069_386_27, -0.228_976_76, 1.159_590_5],
];

/// Matrix by vector multiplication: `m * x` of a 3x3-matrix `m` and a 3-vector `x`.
const fn matvecmul(m: &[[f32; 3]; 3], x: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * x[0] + m[0][1] * x[1] + m[0][2] * x[2],
        m[1][0] * x[0] + m[1][1] * x[1] + m[1][2] * x[2],
        m[2][0] * x[0] + m[2][1] * x[1] + m[2][2] * x[2],
    ]
}

fn pivot(value: f32) -> f32 {
    const KAPPA: f32 = 24389. / 27.;
    // This is EPSILON.cbrt() but that function isn't const (yet)
    const EPSILON_CBRT: f32 = 0.206_896_56;
    if value > EPSILON_CBRT {
        value * value * value
    } else {
        (116. / KAPPA) * value - (16. / KAPPA)
    }
}

impl From<Lab> for RgbVec {
    fn from(value: Lab) -> Self {
        // Adapted to SPIRV from <https://docs.rs/color/0.3.2/src/color/colorspace.rs.html>
        let Lab { l, a, b } = value;
        let f1 = l * (1. / 116.) + (16. / 116.);
        let f0 = a * (1. / 500.) + f1;
        let f2 = f1 - b * (1. / 200.);
        let xyz = [pivot(f0), pivot(f1), pivot(f2)];
        Self(Vec3::from(matvecmul(&LAB_XYZ_TO_SRGB, xyz)))
    }
}

impl From<Lch> for RgbVec {
    fn from(value: Lch) -> Self {
        let lab: Lab = value.into();
        let unclamped: RgbVec = lab.into();
        Self(unclamped.0.clamp(Vec3::ZERO, Vec3::ONE))
    }
}

impl From<Lch> for Hsl {
    fn from(lch: Lch) -> Self {
        let rgb: RgbVec = lch.into();
        rgb.into()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::missing_panics_doc)]
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
}
