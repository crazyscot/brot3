//! Items used by the UI, but not the shader.
//!
//! These are in the lib crate to allow efficient unit testing.
#![cfg(not(spirv))]

use thiserror::Error;

mod dynfmt;
mod messaging;
mod shader_key;
mod shader_variant;
mod state;
mod zoom;

pub use brot3_idl::Exponent;
pub use dynfmt::dynamic_format;
pub use messaging::*;
pub use shader_key::ShaderKey;
pub use shader_variant::ShaderVariant;
pub use state::{UiState, UiStateSaveFile};
pub use zoom::ViewportZoom;

#[derive(Error, Debug)]
#[allow(missing_docs)]
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
    #[error("Unable to convert state data: {0}")]
    StateConversionFailed(String),
}
