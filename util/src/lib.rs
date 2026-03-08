//! Utility and data wrangling code used by the UI and shader crates.
//!
//! This is a separate crate for efficiency of testing.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
