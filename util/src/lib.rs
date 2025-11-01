//! Utility code used by the UI.
//!
//! This is a separate crate for efficiency of testing.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod big_complex;
mod big_vec2;
mod dynfmt;

pub use big_complex::BigComplex;
pub use big_vec2::BigVec2;
pub use dynfmt::dynamic_format;
