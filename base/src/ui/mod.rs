//! Base items used by the UI (not the shader).
//!
//! These are in the base crate for efficiency of testing.
#![cfg(not(target_arch = "spirv"))]

mod exponent;
pub use exponent::Exponent;
