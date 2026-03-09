//! Shared definitions and utilities used throughout brot3.
//!
//! This is a separate crate for efficiency of unit testing, without having to incur
//! the penalty of building spirv-builder and the shader for spirv.

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

pub mod data;
pub mod engine;
pub mod maths;
pub mod util;

/// Complex type used throughout shader
pub type Complex = abels_complex::Complex<f32>;

/// Size of the inspector marker diamond in pixels
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

pub const ESCAPE_THRESHOLD: f32 = 10.0;
pub const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
pub const LOGLOG2_ESCAPE_THRESHOLD: f32 = 1.732_020_9;

#[cfg(not(target_arch = "spirv"))]
mod bignum;
#[cfg(not(target_arch = "spirv"))]
pub mod ui;

#[cfg(not(target_arch = "spirv"))]
pub use bignum::{
    big_complex::BigComplex,
    big_vec2::{BigVec2, fbig_from_str},
};

/// SPIRV `fragment` entrypoint.
/// This does the iteration and rendering work.
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &data::FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &mut [data::PointResult],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    perturbation_reference_points: &[Vec2],
    output: &mut Vec4,
) {
    engine::entrypoints::main_fs(
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
    engine::entrypoints::main_vs(vert_id, out_pos);
}
