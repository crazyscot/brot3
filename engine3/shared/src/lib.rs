#![cfg_attr(target_arch = "spirv", no_std)]

pub mod grid;
pub mod push_constants;
pub use abels_complex as complex;

use glam::{UVec2, uvec2};

pub const GRID_SIZE: UVec2 = uvec2(3840, 2160);
