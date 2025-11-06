#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub type Complex = abels_complex::Complex<f32>;

use core::f32;

#[cfg(not(target_arch = "spirv"))]
use glam::{uvec2, UVec2, Vec2};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{uvec2, UVec2, Vec2};

pub const GRID_SIZE: UVec2 = uvec2(3840, 2160);
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

use bytemuck::{NoUninit, Pod, Zeroable};
use const_default::ConstDefault;

pub use shader_util::{Bool, Size};

#[derive(Copy, Clone, Debug)]
// We only derive NoUninit on non-spirv, because Vec2 is not marked as NoUninint on spirv builds.
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit, Default))]
#[repr(C)]
pub struct FragmentConstants {
    pub flags: Flags,
    pub viewport_translate: Vec2,
    pub viewport_zoom: f32,
    /// window pixel size
    pub size: Size,
    pub algorithm: Algorithm,
    pub max_iter: u32,
    pub exponent: PushExponent,
    pub palette: Palette,
    pub inspector_point_pixel_address: Vec2,
}

bitflags::bitflags! {
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod)]
#[repr(transparent)]
pub struct Flags : u32 {
    const NEEDS_REITERATE = 1 << 0;
    const INSPECTOR_ACTIVE = 1 << 1;

    const _ = !0;
}
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C)]
pub struct Palette {
    pub colourer: Colourer,
    pub style: ColourStyle,
    pub gradient: f32,
    pub offset: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub gamma: f32,
    pub _pad: u32,
}
impl ConstDefault for Palette {
    const DEFAULT: Self = Self {
        colourer: Colourer::DEFAULT,
        style: ColourStyle::DEFAULT,
        // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
        gradient: 1.,
        offset: 0.,
        saturation: 100., // Not available on all palette algorithms
        lightness: 50.,   // Not available on all palette algorithms
        gamma: 1.9,
        _pad: 0,
    };
}
impl Default for Palette {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl Palette {
    pub fn default_with(colourer: Colourer) -> Self {
        Self {
            colourer,
            ..Self::DEFAULT
        }
    }
    pub const MINIMA: Palette = Palette {
        colourer: Colourer::DEFAULT,
        style: ColourStyle::DEFAULT,
        gradient: 0.1,
        offset: -10.0,
        saturation: 0.,
        lightness: 0.,
        gamma: 0.,
        _pad: 0,
    };
    pub const MAXIMA: Palette = Palette {
        colourer: Colourer::DEFAULT,
        style: ColourStyle::DEFAULT,
        gradient: 10.,
        offset: 10.,
        saturation: 100.,
        lightness: 100.,
        gamma: 4.0,
        _pad: 0,
    };
}

/// Raw data from a fractal invocation
///
/// This structure is split into two sub-structs because of the 128MB default limit on data sizes.
/// With a GRID_SIZE of 3840x2160, the default limit allows us 16.18 bytes per grid pixel.
/// Therefore, split our data into shards, each of which is up to 16 bytes in size.
/// If somehow we need to make GRID_SIZE larger, might need to refactor this to split it differently.
///
/// (Yes, we could check the operational capabilities and request more... but that would involve
/// making things dynamic. Not for today.)
#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResult {
    a: PointResultA,
    b: PointResultB,
}

#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResultA {
    /// iteration count
    iters: u32,
    /// fractional part of iteration count (range 0..1)
    iters_fraction: f32,
    /// distance estimate from fractal
    distance: f32,
    /// final angle (argument)
    pub angle: f32,
}

#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResultB {
    /// final complex distance, squared
    pub radius_sqr: f32,
}

// compile time assertion: confirm that neither buffer will runtime fail in wgpu
const _: () = {
    const N_POINTS: usize = (GRID_SIZE.x * GRID_SIZE.y) as usize;
    const LIMIT: usize = 128 * 1024 * 1024; // == wgpu::Limits::max_storage_buffer_binding_size
    assert!(core::mem::size_of::<PointResultA>() * N_POINTS < LIMIT);
    assert!(core::mem::size_of::<PointResultB>() * N_POINTS < LIMIT);
};

impl PointResult {
    // CONSTRUCTORS //////////////////////////////////////////////////////////
    pub fn new_inside(distance: f32, angle: f32, radius_sqr: f32) -> Self {
        Self {
            a: PointResultA {
                iters: u32::MAX,
                iters_fraction: 0.,
                distance,
                angle,
            },
            b: PointResultB { radius_sqr },
        }
    }
    pub fn new_outside(
        iters: u32,
        iters_fraction: f32,
        distance: f32,
        angle: f32,
        radius_sqr: f32,
    ) -> Self {
        Self {
            a: PointResultA {
                iters,
                iters_fraction,
                distance,
                angle,
            },
            b: PointResultB { radius_sqr },
        }
    }
    /// Reconstitutes a `PointResult` from its storage shards
    pub fn join(a: PointResultA, b: PointResultB) -> Self {
        Self { a, b }
    }
    // ACCESSORS ////////////////////////////////////////////////////////////
    pub fn a(&self) -> PointResultA {
        self.a
    }
    pub fn b(&self) -> PointResultB {
        self.b
    }
    /// Iterations
    pub fn iters(&self) -> u32 {
        self.a.iters
    }
    /// Fractional part of iterations (0..1)
    pub fn iters_fraction(&self) -> f32 {
        self.a.iters_fraction
    }
    /// Distance from fractal
    pub fn distance(&self) -> f32 {
        self.a.distance
    }
    /// Final angle
    pub fn angle(&self) -> f32 {
        self.a.angle
    }
    /// Final distance from origin (aka radius or absolute value), squared
    pub fn radius_sqr(&self) -> f32 {
        self.b.radius_sqr
    }
    // COMPUTED ACCESSORS ///////////////////////////////////////////////////
    /// Is this point inside the set? If so, the iterations count is effectively infinite.
    pub fn inside(&self) -> bool {
        self.a.iters == u32::MAX
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[non_exhaustive]
#[repr(u32)]
pub enum NumericType {
    #[default]
    Integer,
    Float,
    Complex,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, NoUninit)]
#[repr(C)]
pub struct PushExponent {
    pub typ: NumericType,
    /// Only used when `typ` is Integer
    pub int: i32,
    /// Used when `typ` is Float or Complex
    pub real: f32,
    /// Only used when `typ` is Complex
    pub imag: f32,
}

impl From<i32> for PushExponent {
    fn from(i: i32) -> Self {
        Self {
            typ: NumericType::Integer,
            int: i,
            ..Default::default()
        }
    }
}

impl From<f32> for PushExponent {
    fn from(f: f32) -> Self {
        Self {
            typ: NumericType::Float,
            real: f,
            ..Default::default()
        }
    }
}

impl From<Complex> for PushExponent {
    fn from(z: Complex) -> Self {
        Self {
            typ: NumericType::Float,
            real: z.re,
            imag: z.im,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
pub enum Algorithm {
    #[default]
    Mandelbrot,
    Mandeldrop,
    Mandelbar,
    BurningShip,
    Celtic,
    Variant,
    BirdOfPrey,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
pub enum Colourer {
    #[default]
    LogRainbow,
    SqrtRainbow,
    WhiteFade,
    BlackFade,
    OneLoneCoder,
    LchGradient,
    Monochrome,
}

impl ConstDefault for Colourer {
    const DEFAULT: Self = Self::LogRainbow;
}

macro_rules! incrementable {
    ($enum:ty) => {
        #[cfg(not(target_arch = "spirv"))]
        impl core::ops::Add<i32> for $enum {
            type Output = Self;

            fn add(self, delta: i32) -> Self::Output {
                use num_traits::FromPrimitive as _;
                use num_traits::ToPrimitive as _;
                use strum::VariantArray as _;
                let n = Self::VARIANTS.len() as i32;
                let mut i = self.to_i32().unwrap_or_default() + delta;
                i = i.rem_euclid(n);
                Self::from_i32(i).unwrap()
            }
        }
        #[cfg(not(target_arch = "spirv"))]
        impl core::ops::AddAssign<i32> for $enum {
            fn add_assign(&mut self, delta: i32) {
                let t = *self + delta;
                *self = t;
            }
        }
    };
}
incrementable!(Colourer);
incrementable!(Algorithm);

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
pub enum ColourStyle {
    #[default]
    ContinuousDwell,
    EscapeTime,
}

impl ConstDefault for ColourStyle {
    const DEFAULT: Self = Self::ContinuousDwell;
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use pretty_assertions::assert_eq;
    #[test]
    fn increment() {
        use super::Colourer;
        let mut c = Colourer::LogRainbow;
        c += 1;
        assert_eq!(c, Colourer::SqrtRainbow);
    }
}
