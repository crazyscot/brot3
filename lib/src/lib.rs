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
use easy_cast::Cast as _;
/// Local glam re-exports for convenience
pub(crate) use spirv_std::glam::{
    UVec2, UVec3, Vec2, Vec3, Vec3Swizzles as _, Vec4, f32, uvec2, uvec3, vec2, vec3,
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
use spirv_std::glam::Vec4Swizzles as _;

use crate::{
    data::{Flags, FragmentConstants, PointResult},
    util::{GridRef, GridRefMut, GridShared, PackedRgba8, RgbVec},
};

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
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5); we
    // ignore depth & 1/w
    let coord = frag_coord.xy();
    let coord_int = coord.as_uvec2();

    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();

    // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;

    let cache = GridRef::new(constants.buffer_size.as_uvec2(), grid);
    let cacheable = cache.address_valid(coord_int);

    let render_data = if !cacheable {
        engine::render(constants, complex_offset, perturbation_reference_points)
    } else if constants.flags.contains(Flags::NEEDS_REITERATE) {
        let render_data = engine::render(constants, complex_offset, perturbation_reference_points);
        let mut cache = GridRefMut::new(constants.buffer_size.as_uvec2(), grid);
        cache.set(coord_int, render_data);
        render_data
    } else {
        // it's cacheable and cached
        cache.get(coord_int)
    };

    let mut colour = engine::colour_data(render_data, constants, pixel_spacing);

    // Draw the inspector marker
    if constants.flags.contains(Flags::INSPECTOR_ACTIVE) {
        // New York distance from the reference point draws a diamond shape
        let dist = engine::new_york_distance(constants.inspector_point_pixel_address, coord);
        if dist < INSPECTOR_MARKER_SIZE * 0.667 {
            // TODO Do something better here? Change pixels underneath?
            colour = RgbVec::BLACK;
        } else if dist < INSPECTOR_MARKER_SIZE {
            colour = RgbVec::WHITE;
        }
    }

    *output = colour.0.extend(1.0);
}

/// SPIRV `vertex` entrypoint.
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2).cast(), (vert_id & 2).cast());
    // uv expresses the cycle: (0,0) (2,0) (0,2) (2,2)
    let pos = 2.0 * uv - Vec2::ONE;
    // pos expresses the cycle: (-1,-1) (3,-1) (-1,3) (3,3)

    *out_pos = pos.extend(0.0).extend(1.0);
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
    // Input gid is the pixel address
    let coord_int = gid.xy();
    let coord = coord_int.as_vec2();

    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();

    // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;

    let render_data = engine::render(constants, complex_offset, perturbation_reference_points);
    let colour = engine::colour_data(render_data, constants, pixel_spacing);

    // no inspector marker in compute shader

    let mut matrix = GridRefMut::new(constants.size.as_uvec2(), pixels);
    matrix.set(gid.xy(), PackedRgba8::from(colour).0);
}
