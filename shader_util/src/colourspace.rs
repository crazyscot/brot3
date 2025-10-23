//! GPU-friendly colour space conversions

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

/// RGB pixel format data. Each component is in the range (0.0, 1.0)
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

#[derive(Clone, Copy, Debug)]
/// HSL colour space
pub struct Hsl {
    /// Hue in degrees (range 0-360)
    h: f32,
    /// Saturation (range 0-100)
    s: f32,
    /// Lightness (range 0-100; 0=black, 50=fully saturated, 100=white)
    l: f32,
}
impl Hsl {
    #[must_use]
    #[allow(missing_docs)]
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
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

#[derive(Clone, Copy, Debug)]
/// LCH colour space
pub struct Lch {
    /// Lightness
    l: f32,
    /// Chroma
    c: f32,
    /// Hue (degrees)
    h: f32,
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
    /// Lightness
    l: f32,
    /// Red-green
    a: f32,
    /// Yellow-blue
    b: f32,
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
