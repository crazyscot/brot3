#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub use abels_complex as complex;

#[cfg(not(target_arch = "spirv"))]
use glam::{uvec2, UVec2, Vec2};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{uvec2, UVec2, Vec2};

pub const GRID_SIZE: UVec2 = uvec2(3840, 2160);

#[cfg(not(target_arch = "spirv"))]
use bytemuck::NoUninit;

use shader_util::{Bool, Size};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit, Default))]
#[repr(C)]
pub struct FragmentConstants {
    pub viewport_translate: Vec2,
    pub viewport_zoom: f32,
    /// window pixel size
    pub size: Size,
    pub algorithm: Algorithm,
    pub max_iter: u32,
    pub needs_reiterate: Bool,
    pub exponent: PushExponent,
    pub palette: Palette,
    pub fractional_iters: Bool,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct Palette {
    pub colourer: Colourer,
    pub gradient: f32,
    pub offset: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub gamma: f32,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            colourer: Default::default(),
            // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
            gradient: 1.,
            offset: 0.,
            saturation: 100., // Not available on all palette algorithms
            lightness: 50.,   // Not available on all palette algorithms
            gamma: 1.9,
        }
    }
}
impl Palette {
    pub fn default_with(colourer: Colourer) -> Self {
        Self {
            colourer,
            ..Default::default()
        }
    }
    pub const MINIMA: Palette = Palette {
        colourer: Colourer::LogRainbow,
        gradient: 0.1,
        offset: -10.0,
        saturation: 0.,
        lightness: 0.,
        gamma: 0.,
    };
    pub const MAXIMA: Palette = Palette {
        colourer: Colourer::LogRainbow,
        gradient: 10.,
        offset: 10.,
        saturation: 100.,
        lightness: 100.,
        gamma: 4.0,
    };
}

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct PointResult {
    /// iteration count
    pub iters: u32,
    /// fractional iteration count (where available)
    pub fractional_iters: f32,
}

impl PointResult {
    pub fn new(inside: bool, iters: u32, fractional_iters: f32) -> Self {
        if inside {
            Self {
                iters: u32::MAX,
                fractional_iters: u32::MAX as f32,
            }
        } else {
            Self {
                iters,
                fractional_iters,
            }
        }
    }
    pub fn value(&self, fractional: bool) -> f32 {
        if fractional {
            self.fractional_iters
        } else {
            self.iters as f32
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(u32)]
pub enum NumericType {
    #[default]
    Integer,
    Float,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit, Default))]
#[repr(C)]
pub struct PushExponent {
    pub typ: NumericType,
    pub int: i32,
    pub float: f32,
}

impl From<i32> for PushExponent {
    fn from(i: i32) -> Self {
        Self {
            typ: NumericType::Integer,
            int: i,
            float: 0.,
        }
    }
}

impl From<f32> for PushExponent {
    fn from(f: f32) -> Self {
        Self {
            typ: NumericType::Float,
            int: 0,
            float: f,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        NoUninit,
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
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

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        NoUninit,
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
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
