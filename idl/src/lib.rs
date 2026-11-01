//! Data type definitions used by brot3

mod enums;
mod exponent;
mod flags;
mod float;
mod pixel_spacing;
mod point;
mod push_constants;
mod push_exponent;
mod size;
mod vec2;
mod wgsl;

pub use easy_cast;
pub use enums::{
    Algorithm, BoundaryClass, ColourStyle, Colourer, EnumCycle, Modifier, NumericType, RenderMode,
    WgslEnum,
};
pub use exponent::Exponent;
pub use flags::{Flags, WgslFlags};
pub use float::FloatIsNear;
/// Local glam re-exports for convenience
pub(crate) use glam::{UVec2, Vec2, uvec2, vec2};
pub use pixel_spacing::PixelSpacing;
pub use point::PointResult;
pub use push_constants::{FragmentConstants, Palette};
pub use push_exponent::PushExponent;
pub use size::Size;
pub use vec2::WgslVec2;
pub use wgsl::ALL as ALL_WGSL;
