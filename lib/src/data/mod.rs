//! Data type definitions used by brot3

mod enums;
mod point;
mod push_constants;
mod push_exponent;
pub use enums::{Algorithm, BoundaryClass, ColourStyle, Colourer, Modifier};
pub use point::PointResult;
pub use push_constants::{Flags, FragmentConstants, Palette};
pub use push_exponent::{NumericType, PushExponent};
