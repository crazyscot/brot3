#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub type Complex = abels_complex::Complex<f32>;

#[cfg(not(target_arch = "spirv"))]
use glam::{uvec2, UVec2, Vec2};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{uvec2, UVec2, Vec2};

pub const GRID_SIZE: UVec2 = uvec2(3840, 2160);
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

use bytemuck::{NoUninit, Pod, Zeroable};
use const_default::ConstDefault;

use shader_util::Size;

pub mod enums;
use enums::{Algorithm, ColourStyle, Colourer};

use crate::enums::Modifier;
pub mod data;

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
    pub colour_style: ColourStyle,
    pub brightness_style: Modifier,
    pub saturation_style: Modifier,
    pub gradient: f32,
    pub offset: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub gamma: f32,
}
impl ConstDefault for Palette {
    const DEFAULT: Self = Self {
        colourer: Colourer::DEFAULT,
        colour_style: ColourStyle::DEFAULT,
        brightness_style: Modifier::DEFAULT,
        saturation_style: Modifier::DEFAULT,
        // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
        gradient: 1.,
        offset: 0.,
        saturation: 100., // Not available on all palette algorithms
        lightness: 50.,   // Not available on all palette algorithms
        gamma: 1.9,
    };
}
impl Default for Palette {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Palette {
    pub fn with_colourer(mut self, colourer: Colourer) -> Self {
        self.colourer = colourer;
        self
    }
    pub fn with_style(mut self, style: ColourStyle) -> Self {
        self.colour_style = style;
        self
    }
    pub fn with_brightness(mut self, style: Modifier) -> Self {
        self.brightness_style = style;
        self
    }
    pub const MINIMA: Palette = Palette {
        colourer: Colourer::DEFAULT,
        colour_style: ColourStyle::DEFAULT,
        brightness_style: Modifier::DEFAULT,
        saturation_style: Modifier::DEFAULT,
        gradient: 0.1,
        offset: -10.0,
        saturation: 0.,
        lightness: 0.,
        gamma: 0.,
    };
    pub const MAXIMA: Palette = Palette {
        colourer: Colourer::DEFAULT,
        colour_style: ColourStyle::DEFAULT,
        brightness_style: Modifier::DEFAULT,
        saturation_style: Modifier::DEFAULT,
        gradient: 10.,
        offset: 10.,
        saturation: 100.,
        lightness: 100.,
        gamma: 4.0,
    };
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

#[derive(Copy, Clone, Debug, PartialEq, NoUninit)]
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

impl Default for PushExponent {
    fn default() -> Self {
        Self {
            typ: NumericType::Integer,
            int: 2,
            real: 0.,
            imag: 0.,
        }
    }
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
