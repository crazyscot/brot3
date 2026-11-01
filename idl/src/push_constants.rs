//! Structures shared between shader and UI
//!
//! (c) 2025-6 Ross Younger

#![allow(missing_docs)]

use bytemuck::NoUninit;
use const_default::ConstDefault;
use my_wgsl::WgslCompatible;

use crate::{
    Algorithm, ColourStyle, Colourer, Modifier, PushExponent, Size, WgslEnum, WgslFlags, WgslVec2,
};

#[derive(Copy, Clone, Debug, WgslCompatible)]
// We only derive NoUninit on non-spirv, because Vec2 is not marked as NoUninit on spirv builds.
#[cfg_attr(not(spirv), derive(NoUninit))]
#[repr(C)]
#[repr(align(8))]
/// Shader push constants
#[allow(missing_docs)]
pub struct FragmentConstants {
    // Caution! Larger structs must be correctly aligned, hence the seemingly random ordering.
    pub exponent: PushExponent, // 96 bits
    pub flags: WgslFlags,       // u32

    /// window pixel size
    pub viewport_size: Size, // 64 bits
    /// size of pixel cache grid we've allocated
    pub buffer_size: Size, // 64 bits
    pub inspector_point_pixel_address: WgslVec2, // 64 bits
    pub viewport_translate: WgslVec2,            // 64 bits

    pub viewport_zoom: f32,
    pub algorithm: WgslEnum<Algorithm>, // u32
    pub max_iter: u32,
    pub palette: Palette, // u32
}

// compile time assertion: confirm that push constants will fit into the size that e-s-r requests
const _: () = {
    assert!(core::mem::size_of::<FragmentConstants>() < 128);
};

#[allow(missing_docs)]
impl FragmentConstants {
    pub const DEFAULT_MAX_ITER: u32 = 250;
    pub const DEFAULT_ZOOM: f32 = 0.25;
}

impl Default for FragmentConstants {
    /// Caution: The default implementation sets both `viewport_size` and `buffer_size` to (0,0).
    fn default() -> Self {
        Self {
            flags: WgslFlags::default(),
            viewport_translate: WgslVec2::default(),
            viewport_zoom: Self::DEFAULT_ZOOM,
            viewport_size: Size::new(0, 0),
            buffer_size: Size::new(0, 0),
            max_iter: Self::DEFAULT_MAX_ITER,
            algorithm: Algorithm::default().into(),
            exponent: PushExponent::default(),
            palette: Palette::default(),
            inspector_point_pixel_address: WgslVec2::default(),
        }
    }
}

impl FragmentConstants {
    #[must_use]
    pub fn pixel_spacing(&self) -> f32 {
        use crate::PixelSpacing as _;
        self.viewport_zoom.pixel_spacing(self.viewport_size.height)
    }
}

#[derive(Copy, Clone, Debug, NoUninit, PartialEq, WgslCompatible)]
#[cfg_attr(not(spirv), derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
/// Colouring palette selection and parameters
pub struct Palette {
    pub colourer: WgslEnum<Colourer>,
    pub colour_style: WgslEnum<ColourStyle>,
    pub brightness_style: WgslEnum<Modifier>,
    pub gradient: f32,
    pub offset: f32,
}
impl ConstDefault for Palette {
    const DEFAULT: Self = Self {
        colourer: WgslEnum(Colourer::Neon2),
        colour_style: WgslEnum(ColourStyle::DEFAULT),
        brightness_style: WgslEnum(Modifier::DEFAULT),
        // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
        gradient: 1.,
        offset: 0.,
    };
}
impl Default for Palette {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Palette {
    pub const MAXIMA: Palette = Palette {
        colourer: WgslEnum(Colourer::DEFAULT),
        colour_style: WgslEnum(ColourStyle::DEFAULT),
        brightness_style: WgslEnum(Modifier::DEFAULT),
        gradient: 10.,
        offset: 10.,
    };
    pub const MINIMA: Palette = Palette {
        colourer: WgslEnum(Colourer::DEFAULT),
        colour_style: WgslEnum(ColourStyle::DEFAULT),
        brightness_style: WgslEnum(Modifier::DEFAULT),
        gradient: 0.1,
        offset: -10.0,
    };

    #[must_use]
    pub fn with_colourer(mut self, colourer: Colourer) -> Self {
        self.colourer = colourer.into();
        self
    }

    #[must_use]
    pub fn with_style(mut self, style: ColourStyle) -> Self {
        self.colour_style = style.into();
        self
    }

    #[must_use]
    pub fn with_brightness(mut self, style: Modifier) -> Self {
        self.brightness_style = style.into();
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {

    use float_eq::assert_float_eq;

    use crate::{ColourStyle, Flags, Palette, WgslEnum};

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
        use crate::PixelSpacing as _;

        assert_float_eq!(
            12345.0f64.pixel_spacing(1920),
            12345.0f32.pixel_spacing(1920).into(),
            abs <= 0.00001
        );
    }

    #[test]
    fn construct_palette() {
        let p = Palette::default().with_style(ColourStyle::Discrete);
        assert_eq!(p.colour_style, WgslEnum(ColourStyle::Discrete));
    }
}
