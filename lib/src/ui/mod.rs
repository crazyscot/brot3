//! Items used by the UI, but not the shader.
//!
//! These are in the lib crate to allow efficient unit testing.
#![cfg(not(target_arch = "spirv"))]

mod dynfmt;
mod exponent;
mod zoom;

pub use dynfmt::dynamic_format;
pub use exponent::Exponent;
pub use zoom::ViewportZoom;
