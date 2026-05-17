//! Structures shared between shader and UI
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

use bytemuck::{NoUninit, Pod, Zeroable};
use const_default::ConstDefault;
use spirv_std::glam::Vec2;

use super::{Algorithm, ColourStyle, Colourer, Modifier, PushExponent};
use crate::util::Size;

#[derive(Copy, Clone, Debug)]
// We only derive NoUninit on non-spirv, because Vec2 is not marked as NoUninit on spirv builds.
#[cfg_attr(not(spirv), derive(NoUninit))]
#[repr(C)]
/// Shader push constants
#[allow(missing_docs)]
pub struct FragmentConstants {
    // Caution! Larger structs must be correctly aligned, hence the seemingly random ordering.
    pub exponent: PushExponent, // 96 bits
    pub flags: Flags,           // u32

    /// window pixel size
    pub size: Size, // 64 bits
    /// size of pixel cache grid we've allocated
    pub buffer_size: Size, // 64 bits
    pub inspector_point_pixel_address: Vec2, // 64 bits
    pub viewport_translate: Vec2,            // 64 bits

    pub viewport_zoom: f32,
    pub algorithm: Algorithm, // u32
    pub max_iter: u32,
    pub palette: Palette, // u32
}

// compile time assertion: confirm that push constants will fit into the size that e-s-r requests
const _: () = {
    assert!(core::mem::size_of::<FragmentConstants>() < 128);
};

#[allow(missing_docs)]
impl FragmentConstants {
    pub(crate) const DEFAULT_MAX_ITER: u32 = 250;
    pub(crate) const DEFAULT_ZOOM: f32 = 0.25;
}

impl Default for FragmentConstants {
    /// Caution: The default implementation sets both `size` and `buffer_size` to (0,0).
    fn default() -> Self {
        Self {
            flags: Flags::default(),
            viewport_translate: Vec2::ZERO,
            viewport_zoom: Self::DEFAULT_ZOOM,
            size: Size::new(0, 0),
            buffer_size: Size::new(0, 0),
            max_iter: Self::DEFAULT_MAX_ITER,
            algorithm: Algorithm::default(),
            exponent: PushExponent::default(),
            palette: Palette::default(),
            inspector_point_pixel_address: Vec2::default(),
        }
    }
}

bitflags::bitflags! {
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod, PartialEq)]
#[repr(transparent)]
#[allow(missing_docs)]
/// Flag bits for shader operation, packed into a u32 in the push constants
pub struct Flags : u32 {
    const NEEDS_REITERATE = 1 << 0;
    const INSPECTOR_ACTIVE = 1 << 1;
    const PERTURBATION_MODE = 1 << 2;
    const ITERATION_CULL = 1<<3;
    /// Set if we want to calculate distance estimation information (which is moderately expensive)
    const DISTANCE_ESTIMATE = 1<<4;

    const _ = !0;
}
}

impl Flags {
    /// Conditionally returns a flag value
    #[must_use]
    pub fn flag_if(condition: bool, flag: Flags) -> Flags {
        if condition { flag } else { Flags::empty() }
    }
}

impl FragmentConstants {
    #[must_use]
    pub fn pixel_spacing(&self) -> f32 {
        use crate::engine::PixelSpacing as _;
        self.viewport_zoom.pixel_spacing(self.size.height)
    }
}

#[derive(Copy, Clone, Debug, NoUninit, PartialEq)]
#[cfg_attr(not(spirv), derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
/// Colouring palette selection and parameters
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
        colourer: Colourer::Neon,
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

    #[must_use]
    pub fn with_colourer(mut self, colourer: Colourer) -> Self {
        self.colourer = colourer;
        self
    }

    #[must_use]
    pub fn with_style(mut self, style: ColourStyle) -> Self {
        self.colour_style = style;
        self
    }

    #[must_use]
    pub fn with_brightness(mut self, style: Modifier) -> Self {
        self.brightness_style = style;
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {

    use float_eq::assert_float_eq;

    use super::{ColourStyle, Flags, Palette};

    #[test]
    fn flags_if() {
        assert_eq!(
            Flags::flag_if(true, Flags::NEEDS_REITERATE),
            Flags::NEEDS_REITERATE
        );
        assert_eq!(
            Flags::flag_if(false, Flags::NEEDS_REITERATE),
            Flags::empty()
        );
    }

    #[test]
    fn pixel_spacing() {
        use crate::engine::PixelSpacing as _;

        assert_float_eq!(
            12345.0f64.pixel_spacing(1920),
            12345.0f32.pixel_spacing(1920).into(),
            abs <= 0.00001
        );
    }

    #[test]
    fn construct_palette() {
        let p = Palette::default().with_style(ColourStyle::Discrete);
        assert_eq!(p.colour_style, ColourStyle::Discrete);
    }
}
