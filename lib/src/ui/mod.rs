//! Items used by the UI, but not the shader.
//!
//! These are in the lib crate to allow efficient unit testing.
#![cfg(not(target_arch = "spirv"))]

mod dynfmt;
mod exponent;
mod state;
mod zoom;

pub use dynfmt::dynamic_format;
pub use exponent::Exponent;
pub use state::UiState;
pub use zoom::ViewportZoom;

/// The maximum precision to use for `BigVec2` and `BigComplex`.
///
/// dashu uses whatever actual digit size it considers necessary, up to this limit.
/// Larger limits reduce performance in deep zooms, but may improve accuracy.
// TODO figure out what precision is best; do we need to make it dynamic?
pub const BIGNUM_PRECISION_LIMIT: usize = 192;
