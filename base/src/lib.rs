//! Shared definitions and utilities used throughout brot3.
//!
//! This is a separate crate for efficiency of testing, and how the dependencies worked out.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod enums;
mod push_exponent;

#[cfg(not(target_arch = "spirv"))]
mod big;
#[cfg(not(target_arch = "spirv"))]
pub mod ui;

#[cfg(not(target_arch = "spirv"))]
pub use big::{
    big_complex::BigComplex,
    big_vec2::{BigVec2, fbig_from_str},
};
pub use push_exponent::{NumericType, PushExponent};
