//! Save and Load support
// (c) 2026 Ross Younger

use std::{
    fs::File,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use brot3_lib::{
    data::{Flags, FragmentConstants, PointResult, RenderMode},
    ui::{Error as LibError, UiState},
};
use easy_cast::CastApprox;
use glam::{Vec2, Vec4, uvec2, vec4};
use rayon::prelude::*;
use thiserror::Error;

use crate::compute::ComputeController;

/// The error type used by this module
#[derive(Error, Debug, strum::EnumIs)]
#[allow(missing_docs)]
pub enum LoadSaveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG encoding error: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[error("{0}")]
    Lib(#[from] LibError),
    #[error("Some pixels failed to render. The saved image may have gaps where this occurred.")]
    PartialRenderFailure,
    #[error("Compute shader controller failed: {0}")]
    ComputeController(#[from] crate::compute::ComputeControllerError),
}

pub(crate) fn do_save_image(
    path: &Path,
    state: &UiState,
    perturbation_points: &[Vec2],
    mode: RenderMode,
) -> Result<(), LoadSaveError> {
    let mut constants = FragmentConstants::from(state);
    constants.flags |= Flags::NEEDS_REITERATE;
    constants.buffer_size = uvec2(0, 0).into();
    log::debug!(
        "Saving image to {} with constants: {constants:?}",
        path.display()
    );
    if !perturbation_points.is_empty() {
        constants.flags |= Flags::PERTURBATION_MODE;
    }
    let start = Instant::now();
    let (pixels, partial_failure) = match mode {
        RenderMode::CpuSingleThreaded => render_cpu(&constants, perturbation_points, false),
        RenderMode::CpuParallel => render_cpu(&constants, perturbation_points, true),
        RenderMode::Gpu => render_gpu(&constants, perturbation_points)
            .inspect_err(|e| log::warn!("Failed to render on GPU, falling back to CPU: {e}"))
            .map_or_else(
                |_| render_cpu(&constants, perturbation_points, true),
                |vec| (vec, false),
            ),
    };
    let duration = start.elapsed();
    log::debug!("Rendered image in {duration:?}");

    let pngstart = Instant::now();

    assert_eq!(constants.size, state.viewport_size.into());
    write_png(path, state, &pixels)?;
    log::debug!("Converted to PNG in {:?}", pngstart.elapsed());
    if partial_failure {
        return Err(LoadSaveError::PartialRenderFailure);
    }
    Ok(())
}

fn render_cpu(
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

    let render_chunk = |(chunk_idx, chunk): (usize, &mut [u8])| {
        let byte_offset = chunk_idx * chunk_bytes;
        let start_pixel = byte_offset / 4;
        let mut y = start_pixel / width;
        let mut x = start_pixel % width;

        for i in (0..chunk.len()).step_by(4) {
            let result = std::panic::catch_unwind(|| {
                let mut grid = [PointResult::default()];
                let mut pixel = Vec4::default();

                let frag_coord = vec4(x.cast_approx(), y.cast_approx(), 0.0, 0.0);
                brot3_lib::main_fs(
                    frag_coord,
                    constants,
                    &mut grid,
                    perturbation_points,
                    &mut pixel,
                );
                pixel
            });
            let pixel = if let Ok(p) = result {
                p
            } else {
                log::debug!("Panic at pixel ({x}, {y})");
                failure.store(true, Ordering::Relaxed);
                Vec4::ZERO
            };

            let bytes = (pixel * 255.0).as_u8vec4().to_array();
            chunk[i..i + 4].copy_from_slice(&bytes);

            // Move to next pixel
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
    };

    if parallel {
        pixels
            .par_chunks_mut(chunk_bytes)
            .enumerate()
            .for_each(render_chunk);
    } else {
        pixels
            .chunks_mut(chunk_bytes)
            .enumerate()
            .for_each(&render_chunk);
    }
    (pixels, failure.load(Ordering::Relaxed))
}

fn render_gpu(
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
) -> Result<Vec<u8>, LoadSaveError> {
    let render_size = constants.size.into();
    let mut controller = ComputeController::new(render_size, 1)?;
    let mut frame_data = Vec::with_capacity(render_size.element_product() as usize);
    let times = controller.run(
        *constants,
        render_size.extend(1),
        1,
        perturbation_points,
        |rgba| {
            frame_data.clear();
            frame_data.extend_from_slice(rgba);
        },
    )?;
    if times.len() == 4 {
        use itertools::Itertools as _;
        // The compute controller provides 4 timestamps: start, start of compute, completion of
        // compute, teardown. These in turn can be resolved into phases: setup, compute,
        // teardown.
        let overall = Duration::from_nanos(times.last().unwrap() - times[0]);
        let deltas = times
            .into_iter()
            .tuple_windows()
            .map(|(start, end)| Duration::from_nanos(end - start))
            .collect::<Vec<_>>();
        log::debug!(
            "GPU timing: setup {:?}, compute {:?}, teardown {:?}, overall {:?}",
            deltas[0],
            deltas[1],
            deltas[2],
            overall
        );
    } else if !times.is_empty() {
        log::warn!(
            "Expected 4 timestamps from compute controller, got {}",
            times.len()
        );
    } // else ignore: not all devices support timestamp queries, and the controller will simply return an empty Vec in that case.
    Ok(frame_data)
}

/// Writes the given pixel data to a PNG file, embedding metadata about the UI state and
/// software version.
pub fn write_png(path: &Path, state: &UiState, pixels: &[u8]) -> Result<(), LoadSaveError> {
    let mut encoder = png::Encoder::new(
        File::create(path)?,
        state.viewport_size.x,
        state.viewport_size.y,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
    encoder.add_text_chunk("comment".to_string(), state.display_string(' '))?;
    serde_json::to_string(&state)
        .ok()
        .and_then(|s| encoder.add_text_chunk("uistate".to_string(), s).ok())
        .unwrap_or_else(|| {
            log::warn!("Failed to serialize UI state for embedding in PNG metadata");
        });
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    Ok(())
}
