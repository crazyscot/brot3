//! Helper types for GPU shaders

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod grid;
pub use grid::*;

/// Re-exported from [`glam`].
pub use glam::UVec2;
use glam::{Vec2, uvec2, vec2};
