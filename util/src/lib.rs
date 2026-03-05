//! Utility and data wrangling code used by the UI and shader crates.
//!
//! This is a separate crate for efficiency of testing.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod dprintln;

#[cfg(not(target_arch = "spirv"))]
mod big;
#[cfg(not(target_arch = "spirv"))]
pub use big::{big_complex::BigComplex, big_vec2::BigVec2};

#[cfg(not(target_arch = "spirv"))]
mod dynfmt;
#[cfg(not(target_arch = "spirv"))]
pub use dynfmt::dynamic_format;

#[cfg(not(target_arch = "spirv"))]
mod exponent;
#[cfg(not(target_arch = "spirv"))]
pub use exponent::Exponent;

mod push_exponent;
pub use push_exponent::{NumericType, PushExponent};
