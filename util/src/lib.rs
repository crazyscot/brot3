//! Utility code used by the UI.
//!
//! This is a separate crate for efficiency of testing.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod dprintln;

mod dynfmt;
pub use dynfmt::dynamic_format;
