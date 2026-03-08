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

pub use spirv_std::glam::{DVec2, UVec2, Vec2, Vec3, Vec4, f32, uvec2, vec2, vec3, vec4};
#[allow(unused_imports)] // Some are reused in some configurations
use spirv_std::spirv;

pub mod colour;
pub mod colourspace;
pub mod data;
pub mod entrypoints;
pub mod enums;
pub mod exponentiation;
pub mod fractal;
pub mod grid;
mod pixels;
pub mod push_constants;
mod size;

use data::PointResult;
pub use enums::{Algorithm, ColourStyle, Colourer};
pub use pixels::PixelSpacing;
pub use push_constants::{Flags, FragmentConstants, INSPECTOR_MARKER_SIZE, Palette};
pub use size::Size;

/// Complex type used throughout shader
pub type Complex = abels_complex::Complex<f32>;

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
