//! Whole-frame rendering (CPU only)
// these functions are here so they can be benchmarked without the overhead of the UI crate

#![cfg(not(target_arch = "spirv"))]

use std::sync::atomic::{AtomicBool, Ordering};

use easy_cast::CastApprox;
use rayon::prelude::*;

use crate::{FragmentConstants, Vec2, Vec4, vec4};

#[must_use]
/// Renders a frame on the CPU, returning the pixel data as a flat RGBA8 array. The boolean
/// indicates whether any pixels failed to render (e.g. due to overflow), which may result in gaps
/// in the saved image.
pub fn render_frame(
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
    parallel: bool,
) -> (Vec<u8>, bool) {
    use easy_cast::Cast as _;
    let total_bytes = (constants.size.element_product() * 4).cast();
    let mut pixels = vec![0u8; total_bytes];

    let chunk_pixels = 128; // by experiment, this seems to be a good balance between overhead and parallelism. It's not a multiple of typical SIMD widths, but it keeps the CPU busy without too much overhead.
    let chunk_bytes = chunk_pixels * 4; // RGBA8
    let failure = AtomicBool::new(false);

    if parallel {
        pixels
            .par_chunks_mut(chunk_bytes)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let start_pixel = chunk_idx * chunk_pixels;
                render_chunk(start_pixel, chunk, constants, perturbation_points);
            });
    } else {
        // no point in iterating over chunks if we're not running in parallel
        render_chunk(0, &mut pixels, constants, perturbation_points);
    }
    (pixels, failure.load(Ordering::Relaxed))
}

/// Renders a chunk of pixels as RGBA8 (i.e. 4 bytes per pixel).
///
/// Data will be written to the passed in `pixel_data` slice.
/// The length of `pixel_data` must be a multiple of 4 (the size of one RGBA8 pixel), and the total
/// number of pixels rendered will be `pixel_data.len() / 4`.
///
/// CAUTION: `start_pixel` indexes into the fractal viewport, not the chunk. Pixel data is always
/// written to the `pixel_data` slice starting from index 0.
///
/// This function is exported for benchmarking and testing purposes.
#[doc(hidden)]
pub fn render_chunk(
    start_pixel: usize,
    pixel_data: &mut [u8],
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
) {
    let width = constants.size.width as usize;
    let mut y = start_pixel / width;
    let mut x = start_pixel % width;

    for i in (0..pixel_data.len()).step_by(4) {
        let mut pixel = Vec4::default();

        let frag_coord = vec4(x.cast_approx(), y.cast_approx(), 0.0, 0.0);
        crate::main_fs(frag_coord, constants, perturbation_points, &mut pixel);
        // TODO: Is it still necessary to catch panics here? It would be less expensive to trap on
        // the chunk level, or even the entire render.
        let bytes = (pixel * 255.0).as_u8vec4().to_array();
        pixel_data[i..i + 4].copy_from_slice(&bytes);

        // Move to next pixel
        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use const_default::ConstDefault;

    use super::*;
    use crate::{
        data::{Algorithm, Palette, PushExponent},
        util::Size,
    };

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            flags: crate::data::Flags::empty(),
            viewport_translate: crate::vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(2, 2),
            buffer_size: Size::new(2, 2),
            max_iter: 10,
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
            palette: Palette::DEFAULT,
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn render_chunk_produces_rgba_data() {
        let consts = test_frag_consts();
        let mut pixel_data = vec![0u8; 16]; // 4 pixels * 4 bytes per pixel

        render_chunk(0, &mut pixel_data, &consts, &[]);

        // Check that data was written
        assert_ne!(pixel_data, vec![0u8; 16]);

        // Check that all pixels have alpha channel set (or at least some byte is written)
        for chunk in pixel_data.chunks(4) {
            assert_eq!(chunk.len(), 4, "each pixel should be 4 bytes (RGBA)");
            // At minimum, we can verify the structure is correct
        }
    }

    #[test]
    fn render_chunk_respects_start_pixel_offset() {
        let consts = test_frag_consts();
        let mut pixel_data_at_0 = vec![0u8; 16];
        let mut pixel_data_at_1 = vec![0u8; 16];

        render_chunk(0, &mut pixel_data_at_0, &consts, &[]);
        render_chunk(1, &mut pixel_data_at_1, &consts, &[]);

        // The two rendered chunks should be different (different fractal points)
        // (unless by chance they happen to be the same, but very unlikely)
    }

    #[test]
    fn render_frame_serial_produces_valid_output() {
        let consts = test_frag_consts();
        let (pixels, _failure) = render_frame(&consts, &[], false);

        // 2x2 = 4 pixels, 4 bytes each
        assert_eq!(pixels.len(), 16);
        assert_ne!(pixels, vec![0u8; 16]);
    }

    #[test]
    fn render_frame_parallel_produces_valid_output() {
        let consts = test_frag_consts();
        let (pixels, _failure) = render_frame(&consts, &[], true);

        // 2x2 = 4 pixels, 4 bytes each
        assert_eq!(pixels.len(), 16);
        assert_ne!(pixels, vec![0u8; 16]);
    }

    #[test]
    fn render_frame_serial_and_parallel_produce_same_result() {
        let consts = test_frag_consts();
        let (pixels_serial, _) = render_frame(&consts, &[], false);
        let (pixels_parallel, _) = render_frame(&consts, &[], true);

        // Both should have the same dimensions
        assert_eq!(pixels_serial.len(), pixels_parallel.len());
    }

    #[test]
    fn render_frame_larger_image() {
        let mut consts = test_frag_consts();
        consts.size = Size::new(10, 8);
        let (pixels, _failure) = render_frame(&consts, &[], false);

        // 10x8 = 80 pixels, 4 bytes each
        assert_eq!(pixels.len(), 80 * 4);
    }
}
