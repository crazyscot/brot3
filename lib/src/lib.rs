//! Shared definitions and utilities used throughout brot3.
//!
//! This is a separate crate for efficiency of testing, and how the dependencies worked out.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod enums;
mod float;
mod pixels;
mod push_exponent;

#[allow(unused)]
pub(crate) use float::FloatIsNear;
pub use pixels::PixelSpacing;
pub use push_exponent::{NumericType, PushExponent};

#[cfg(not(target_arch = "spirv"))]
mod bignum;
#[cfg(not(target_arch = "spirv"))]
pub mod ui;

#[cfg(not(target_arch = "spirv"))]
pub use bignum::{
    big_complex::BigComplex,
    big_vec2::{BigVec2, fbig_from_str},
};
