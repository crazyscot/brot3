//! Structures shared between shader and UI

#![allow(missing_docs)]

use bytemuck::{NoUninit, Pod, Zeroable};
use const_default::ConstDefault;
use spirv_std::glam::{UVec2, Vec2, uvec2};
pub(crate) use util::{NumericType, PushExponent};

use crate::{
    ColourStyle, Colourer, Size,
    enums::{Algorithm, Modifier},
};

/// Size of the inspector marker diamond in pixels
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

pub const ESCAPE_THRESHOLD: f32 = 10.0;
pub const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
pub const LOGLOG2_ESCAPE_THRESHOLD: f32 = 1.732_020_9;

#[cfg(not(target_arch = "spirv"))]
fn compile_time_checks() {
    build_assert::build_assert!(float_eq::float_eq!(
        2.0f32.powf(2.0f32.powf(LOGLOG2_ESCAPE_THRESHOLD)),
        ESCAPE_THRESHOLD,
        abs <= 0.0001
    ));
}

#[derive(Copy, Clone, Debug)]
// We only derive NoUninit on non-spirv, because Vec2 is not marked as NoUninit on spirv builds.
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
/// Shader push constants
#[allow(missing_docs)]
pub struct FragmentConstants {
    // Caution! Larger structs must be correctly aligned, hence the random ordering.
    pub exponent: PushExponent, // 128 bits

    /// window pixel size
    pub size: Size, // 64 bits
    /// size of pixel cache grid we've allocated
    pub buffer_size: Size, // 64 bits
    pub inspector_point_pixel_address: Vec2, // 64 bits
    pub viewport_translate: Vec2,            // 64 bits

    pub flags: Flags, // u32
    pub viewport_zoom: f32,
    pub algorithm: Algorithm, // u32
    pub max_iter: u32,
    pub palette: Palette, // u32
    // number of points in the perturbation buffer
    pub n_reference_points: u32,
}

// compile time assertion: confirm that push constants will fit into the size that e-s-r requests
const _: () = {
    assert!(core::mem::size_of::<FragmentConstants>() < 128);
};

#[allow(missing_docs)]
impl FragmentConstants {
    pub const DEFAULT_MAX_ITER: u32 = 250;
    pub const DEFAULT_SIZE: UVec2 = uvec2(800, 600);
    pub const DEFAULT_ZOOM: f32 = 0.25;
    /// Conversion factor applied to `viewport_zoom` whenever it's presented to a human
    pub const UI_ZOOM_FACTOR: f32 = 4.0;
}

impl Default for FragmentConstants {
    fn default() -> Self {
        #[cfg(not(target_arch = "spirv"))]
        compile_time_checks();
        Self {
            flags: Flags::default(),
            viewport_translate: Vec2::ZERO,
            viewport_zoom: Self::DEFAULT_ZOOM,
            size: Self::DEFAULT_SIZE.into(),
            buffer_size: Self::DEFAULT_SIZE.into(),
            max_iter: Self::DEFAULT_MAX_ITER,
            algorithm: Algorithm::default(),
            exponent: PushExponent::default(),
            palette: Palette::default(),
            inspector_point_pixel_address: Vec2::default(),
            n_reference_points: 0,
        }
    }
}

bitflags::bitflags! {
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod, PartialEq)]
#[repr(transparent)]
#[allow(missing_docs)]
pub struct Flags : u32 {
    const NEEDS_REITERATE = 1 << 0;
    const INSPECTOR_ACTIVE = 1 << 1;
    const PERTURBATION_MODE = 1 << 2;
    const ITERATION_CULL = 1<<3;

    const _ = !0;
}
}

/// Conditionally returns a flag value
#[must_use]
pub fn flag_if(condition: bool, flag: Flags) -> Flags {
    if condition { flag } else { Flags::empty() }
}

impl FragmentConstants {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn pixel_spacing_f32(height: u32, zoom: f32) -> f32 {
        1.0 / (height as f32 * zoom)
    }

    #[cfg(not(target_arch = "spirv"))]
    #[must_use]
    pub fn pixel_spacing_f64(height: u32, zoom: f64) -> f64 {
        1.0 / (f64::from(height) * zoom)
    }

    #[must_use]
    pub fn pixel_spacing(&self) -> f32 {
        Self::pixel_spacing_f32(self.size.height, self.viewport_zoom)
    }

    #[cfg(not(target_arch = "spirv"))]
    #[must_use]
    pub fn display_string(&self) -> String {
        format!(
            "{alg}_({x},{y})_z{zoom:.3e}_max{max_iter}_exp{exp}_{colourer:?}",
            alg = self.algorithm,
            x = self.viewport_translate.x,
            y = self.viewport_translate.y,
            zoom = (1.0 / self.viewport_zoom) / Self::UI_ZOOM_FACTOR,
            max_iter = self.max_iter,
            exp = match self.exponent.typ {
                NumericType::Integer => self.exponent.int.to_string(),
                NumericType::Float => format!("{:.3}", self.exponent.real),
                NumericType::Complex =>
                    format!("{:.3}+{:.3}i", self.exponent.real, self.exponent.imag),
                _ => unimplemented!(),
            },
            colourer = self.palette.colourer,
        )
    }
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(serde::Serialize, serde::Deserialize)
)]
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
#[allow(clippy::missing_panics_doc)] // I shouldn't need to write this here, but rust-analyzer is confused.
mod tests {

    use float_eq::assert_float_eq;

    use super::{Flags, flag_if};
    use crate::{ColourStyle, FragmentConstants, Palette};

    #[test]
    fn flags_if() {
        assert_eq!(
            flag_if(true, Flags::NEEDS_REITERATE),
            Flags::NEEDS_REITERATE
        );
        assert_eq!(flag_if(false, Flags::NEEDS_REITERATE), Flags::empty());
    }

    #[test]
    fn pixel_spacing() {
        assert_float_eq!(
            f64::from(FragmentConstants::pixel_spacing_f32(1920, 12345.0)),
            FragmentConstants::pixel_spacing_f64(1920, 12345.0),
            abs <= 0.00001
        );
    }

    #[test]
    fn construct_palette() {
        let p = Palette::default().with_style(ColourStyle::Discrete);
        assert_eq!(p.colour_style, ColourStyle::Discrete);
    }
}
