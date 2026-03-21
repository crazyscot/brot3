//! Items used by the UI, but not the shader.
//!
//! These are in the lib crate to allow efficient unit testing.
#![cfg(not(spirv))]

use thiserror::Error;

mod dynfmt;
mod exponent;
mod messaging;
mod state;
mod zoom;

pub use dynfmt::dynamic_format;
pub use exponent::Exponent;
pub use messaging::*;
pub use state::{UiState, UiStateSaveFile};
pub use zoom::ViewportZoom;

/// The maximum precision to use for `BigVec2` and `BigComplex`.
///
/// dashu uses whatever actual digit size it considers necessary, up to this limit.
/// Larger limits reduce performance in deep zooms, but may improve accuracy.
// TODO figure out what precision is best; do we need to make it dynamic?
pub const BIGNUM_PRECISION_LIMIT: usize = 192;

#[derive(Error, Debug)]
/// The error type used by [`crate::ui`]
pub enum Error {
    #[error("Unsupported save file version {0}")]
    UnsupportedVersion(u32),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("PNG decoding error: {0}")]
    PngDecode(#[from] png::DecodingError),
    #[error("Unrecognised file format")]
    UnrecognisedFormat,
    #[error("PNG file did not contain usable state data")]
    PngHadNoStateData,
}
