//! Shared definitions and utilities used throughout brot3.
//!
//! This is a separate crate for efficiency of unit testing, without having to incur
//! the penalty of building spirv-builder and the shader for spirv.

#![cfg_attr(spirv, no_std)]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

//! ## Feature flags
#![doc = document_features::document_features!()]

pub use easy_cast;
/// Local glam re-exports for convenience
pub(crate) use glam::{
    DVec2, UVec2, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, dvec2, f32, uvec2, uvec3,
    vec2, vec3, vec4,
};

mod bignum;

pub mod data;
pub mod engine;
pub mod maths;
pub mod ui;
pub mod util;
pub mod wgsl;

pub use bignum::BigComplexExt;
pub use dashu_cmplx::CBig;
pub use dashu_float::FBig;

use crate::data::FragmentConstants;

/// Complex type used throughout shader
pub type Complex = abels_complex::Complex<f32>;

/// Size of the inspector marker diamond in pixels
pub const INSPECTOR_MARKER_SIZE: f32 = 9.;

#[doc(hidden)]
pub const ESCAPE_THRESHOLD: f32 = 10.0;
const ESCAPE_THRESHOLD_SQ: f32 = ESCAPE_THRESHOLD * ESCAPE_THRESHOLD;
const ESCAPE_THRESHOLD_LOGLOG2: f32 = 1.732_020_9; // ESCAPE_THRESHOLD.log2().log2(); alas, log2 isn't a const fn at the moment.

/// SPIRV `fragment` entrypoint.
/// This does the iteration and rendering work.
//#[spirv(fragment)]
pub fn main_fs(
    //#[spirv(frag_coord)]
    frag_coord: Vec4,

    #[cfg(not(feature = "emulate_constants"))]
    //#[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    //#[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,

    //#[spirv(storage_buffer, descriptor_set = 0, binding = 0)]
    perturbation_reference_points: &[Vec2],
    output: &mut Vec4,
) {
    engine::entrypoints::main_fs(frag_coord, constants, perturbation_reference_points, output);
}

/// Number of threads per workgroup for the compute shader
pub const COMPUTE_SHADER_THREADS: UVec3 = uvec3(64, 1, 1);
