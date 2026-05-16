//! Whole-frame rendering (CPU only)
// these functions are here so they can be benchmarked without the overhead of the UI crate

#![cfg(not(target_arch = "spirv"))]

use std::sync::atomic::{AtomicBool, Ordering};

use easy_cast::CastApprox;
use rayon::prelude::*;

use crate::{FragmentConstants, PointResult, Vec2, Vec4, vec4};

#[must_use]
/// Renders a frame on the CPU, returning the pixel data as a flat RGBA8 array. The boolean
/// indicates whether any pixels failed to render (e.g. due to overflow), which may result in gaps
/// in the saved image.
pub fn render_frame(
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
    parallel: bool,
) -> (Vec<u8>, bool) {
    let width = constants.size.width as usize;
    let height = constants.size.height as usize;
    let total_bytes = width * height * 4;
    let mut pixels = vec![0u8; total_bytes];

    let chunk_pixels = 128; // by experiment, this seems to be a good balance between overhead and parallelism. It's not a multiple of typical SIMD widths, but it keeps the CPU busy without too much overhead.
    let chunk_bytes = chunk_pixels * 4; // RGBA8
    let failure = AtomicBool::new(false);

    if parallel {
        pixels
            .par_chunks_mut(chunk_bytes)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                render_chunk(chunk_idx, chunk, constants, perturbation_points);
            });
    } else {
        // no point in iterating over chunks if we're not running in parallel
        render_chunk(0, &mut pixels, constants, perturbation_points);
    }
    (pixels, failure.load(Ordering::Relaxed))
}

/// Renders a chunk of pixels. The chunk index is used to determine the starting pixel coordinates.
/// This function is called in parallel for different chunks, so it should not have side effects or
/// rely on shared state. This function is exported for benchmarking and testing purposes.
#[doc(hidden)]
pub fn render_chunk(
    chunk_idx: usize,
    chunk: &mut [u8],
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
) {
    let width = constants.size.width as usize;
    let byte_offset = chunk_idx * chunk.len();
    let start_pixel = byte_offset / 4;
    let mut y = start_pixel / width;
    let mut x = start_pixel % width;

    for i in (0..chunk.len()).step_by(4) {
        let mut grid = [PointResult::default()];
        let mut pixel = Vec4::default();

        let frag_coord = vec4(x.cast_approx(), y.cast_approx(), 0.0, 0.0);
        crate::main_fs(
            frag_coord,
            constants,
            &mut grid,
            perturbation_points,
            &mut pixel,
        );
        // TODO: Is it still necessary to catch panics here? It would be less expensive to trap on
        // the chunk level, or even the entire render.
        let bytes = (pixel * 255.0).as_u8vec4().to_array();
        chunk[i..i + 4].copy_from_slice(&bytes);

        // Move to next pixel
        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }
}
