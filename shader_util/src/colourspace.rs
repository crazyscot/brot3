//! GPU-friendly colour space representations and conversions

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

use float_eq::float_eq;

/// RGB colour space.
///
/// Each component is in the range (0.0, 1.0).
///
pub use super::Vec3 as Vec3Rgb;

/// Clamps values to the range (0.0, 1.0)
trait Clamp01 {
    /// Clamps a value to the range (0.0, 1.0)
    ///
    /// Returns the clamped value
    fn clamp01(&self) -> Self;
}

impl Clamp01 for Vec3Rgb {
    fn clamp01(&self) -> Self {
        Self {
            x: self.x.clamp01(),
            y: self.y.clamp01(),
            z: self.z.clamp01(),
        }
    }
}
impl Clamp01 for f32 {
    fn clamp01(&self) -> Self {
        self.clamp(0., 1.)
    }
}

/// RGB colour space representation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    /// Red component (range 0..255)
    pub r: f32,
    /// Green component (range 0..255)
    pub g: f32,
    /// Blue component (range 0..255)
    pub b: f32,
}
impl Rgb {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for Vec3Rgb {
    fn from(value: Rgb) -> Self {
        Self {
            x: value.r / 255.0,
            y: value.g / 255.0,
            z: value.b / 255.0,
        }
    }
}
impl From<Vec3Rgb> for Rgb {
    fn from(value: Vec3Rgb) -> Self {
        Self {
            r: value.x * 255.0,
            g: value.y * 255.0,
            b: value.z * 255.0,
        }
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
    #[must_use]
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }
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

impl From<Hsl> for Rgb {
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
        Self {
            r: 255.0 * hue_component(0.),
            g: 255.0 * hue_component(8.),
            b: 255.0 * hue_component(4.),
        }
    }
}

impl From<Rgb> for Hsl {
    fn from(value: Rgb) -> Self {
        #![allow(clippy::many_single_char_names)]
        let r = value.r / 255.0;
        let g = value.g / 255.0;
        let b = value.b / 255.0;
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let l = (max + min) * 0.5;
        if float_eq!(max, min, abs <= 0.000_001) {
            Self {
                h: 0.,
                s: 0.,
                l: l * 100.,
            }
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if float_eq!(max, r, abs <= 0.000_001) {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if float_eq!(max, g, abs <= 0.000_001) {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            } * 60.0;
            Self {
                h,
                s: s * 100.,
                l: l * 100.,
            }
        }
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

const KAPPA: f32 = 24389. / 27.;

impl From<Lab> for Vec3Rgb {
    fn from(value: Lab) -> Self {
        // Adapted to SPIRV from <https://docs.rs/color/0.3.2/src/color/colorspace.rs.html>
        let Lab { l, a, b } = value;
        let f1 = l * (1. / 116.) + (16. / 116.);
        let f0 = a * (1. / 500.) + f1;
        let f2 = f1 - b * (1. / 200.);
        let cbrt = |value| {
            // This is EPSILON.cbrt() but that function isn't const (yet)
            const EPSILON_CBRT: f32 = 0.206_896_56;
            if value > EPSILON_CBRT {
                value * value * value
            } else {
                (116. / KAPPA) * value - (16. / KAPPA)
            }
        };
        let xyz = [cbrt(f0), cbrt(f1), cbrt(f2)];
        matvecmul(&LAB_XYZ_TO_SRGB, xyz).into()
    }
}

impl From<Lch> for Vec3Rgb {
    fn from(value: Lch) -> Self {
        let lab: Lab = value.into();
        let unclamped: Vec3Rgb = lab.into();
        unclamped.clamp01()
    }
}

impl From<Lch> for Hsl {
    fn from(lch: Lch) -> Self {
        let vrgb: Vec3Rgb = lch.into();
        let rgb: Rgb = vrgb.into();
        rgb.into()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Hsl, Rgb};

    fn hsl_rgb_case(hsl: Hsl) {
        let rgb: Rgb = hsl.into();
        assert!(rgb.r >= 0.0 && rgb.r <= 255.0);
        assert!(rgb.g >= 0.0 && rgb.g <= 255.0);
        assert!(rgb.b >= 0.0 && rgb.b <= 255.0);
        let hsl2: Hsl = rgb.into();
        assert_eq!(hsl, hsl2);
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
