//! GPU shader implementing fractal rendering.
//! Can also be called directly on the host (and indeed it is, for the inspector and perturbations
//! mode).

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! ## Feature flags
#![doc = document_features::document_features!()]
#![allow(missing_docs)]
// enable this unstable feature (used in tests):
#![feature(assert_matches)]

use glam::{UVec2, Vec2, Vec3, Vec4, f32, uvec2, vec2, vec3};
pub use spirv_std::glam;
#[allow(unused_imports)] // Some are reused in some configurations
use spirv_std::spirv;

pub mod colour;
pub mod colourspace;
pub mod data;
mod entrypoints;
pub mod exponentiation;
pub mod fractal;
pub mod grid;
pub mod push_constants;
mod size;

use base::data::enums::{self, Algorithm, ColourStyle, Colourer};
use data::PointResult;
#[cfg(test)]
use push_constants::Palette;
use push_constants::{Flags, FragmentConstants};
pub use size::Size;

/// Complex type used throughout shader
pub type Complex = abels_complex::Complex<f32>;

/// Size of the inspector marker diamond in pixels
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

pub const ESCAPE_THRESHOLD: f32 = 10.0;
pub const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
pub const LOGLOG2_ESCAPE_THRESHOLD: f32 = 1.732_020_9;

/// SPIRV `fragment` entrypoint.
/// This does the iteration and rendering work.
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &mut [PointResult],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    perturbation_reference_points: &[Vec2],
    output: &mut Vec4,
) {
    entrypoints::main_fs(
        frag_coord,
        constants,
        grid,
        perturbation_reference_points,
        output,
    );
}

/// SPIRV `vertex` entrypoint.
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    entrypoints::main_vs(vert_id, out_pos);
}
