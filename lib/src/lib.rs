//! Shared definitions and utilities used throughout brot3.
//!
//! This is a separate crate for efficiency of unit testing, without having to incur
//! the penalty of building spirv-builder and the shader for spirv.

#![cfg_attr(spirv, no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

//! ## Feature flags
#![doc = document_features::document_features!()]
#![allow(missing_docs)]

pub use easy_cast;
/// Local glam re-exports for convenience
#[allow(unused_imports)] // Some are reused in tests
pub(crate) use spirv_std::glam::{
    UVec2, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, f32, uvec2, uvec3, uvec4, vec2,
    vec3, vec4,
};
#[cfg(spirv)]
use spirv_std::num_traits::real::Real;
#[allow(unused_imports)] // Some are reused in some configurations
use spirv_std::spirv;

mod bignum;
pub mod data;
pub mod engine;
pub mod maths;
pub mod ui;
pub mod util;

#[cfg(not(spirv))]
pub use bignum::{
    big_complex::{BigComplex, ParseError as BigComplexParseError},
    big_vec2::{BigVec2, fbig_from_str},
};

use crate::data::{FragmentConstants, PointResult};

/// Complex type used throughout shader
pub type Complex = abels_complex::Complex<f32>;

/// Size of the inspector marker diamond in pixels
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

#[doc(hidden)]
pub const ESCAPE_THRESHOLD: f32 = 10.0;
const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;

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

pub const COMPUTE_SHADER_THREADS: UVec3 = uvec3(64, 1, 1);

/// SPIRV compute shader entrypoint
#[spirv(compute(threads(64, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] gid: UVec3,

    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)]
    constants: &FragmentConstants,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] pixels: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    perturbation_reference_points: &[Vec2],
) {
    engine::entrypoints::main_cs(gid, constants, pixels, perturbation_reference_points);
}
