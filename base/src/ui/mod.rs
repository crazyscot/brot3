//! Base items used by the UI (not the shader).
//!
//! These are in the base crate for efficiency of testing.
#![cfg(not(target_arch = "spirv"))]

mod dynfmt;
mod exponent;

pub use dynfmt::dynamic_format;
pub use exponent::Exponent;
