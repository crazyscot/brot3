//! Save Image support
// (c) 2026 Ross Younger

use std::{
    fs::File,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use brot3_lib::{
    data::{Flags, FragmentConstants, PointResult},
    ui::{UiState, UiStateSaveFile},
};
use glam::{Vec2, Vec4, uvec2, vec4};
use rayon::prelude::*;

pub(crate) fn do_save_image(
    path: &std::path::Path,
    mut constants: FragmentConstants,
    state: &UiState,
    perturbation_points: &[Vec2],
) -> anyhow::Result<()> {
    // TODO: We shouldn't need to pass in both state and constants?
    // But they don't quite match up right now. Would have to refactor more of Controller into
    // UiState.
    constants.flags |= Flags::NEEDS_REITERATE;
    constants.buffer_size = uvec2(0, 0).into();
    log::debug!(
        "Saving image to {} with constants: {constants:?}",
        path.display()
    );
    let start = Instant::now();

    let width = constants.size.width as usize;
    let height = constants.size.height as usize;
    let total_bytes = width * height * 4;
    let mut pixels = vec![0u8; total_bytes];

    let chunk_pixels = 128; // by experiment, this seems to be a good balance between overhead and parallelism. It's not a multiple of typical SIMD widths, but it keeps the CPU busy without too much overhead.
    let chunk_bytes = chunk_pixels * 4; // RGBA8
    let failure = AtomicBool::new(false);

    pixels
        .par_chunks_mut(chunk_bytes)
        .enumerate()
        .for_each(|(chunk_idx, chunk)| {
            let byte_offset = chunk_idx * chunk_bytes;
            let start_pixel = byte_offset / 4;
            let mut y = start_pixel / width;
            let mut x = start_pixel % width;

            for i in (0..chunk.len()).step_by(4) {
                let result = std::panic::catch_unwind(|| {
                    let mut grid = [PointResult::default()];
                    let mut pixel = Vec4::default();

                    #[allow(clippy::cast_precision_loss)]
                    let frag_coord = vec4(x as f32, y as f32, 0.0, 0.0);
                    brot3_lib::main_fs(
                        frag_coord,
                        &constants,
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
        });

    let duration = start.elapsed();
    log::debug!("Rendered image in {duration:?}");

    let pngstart = Instant::now();
    let mut encoder = png::Encoder::new(
        File::create(path)?,
        constants.size.width,
        constants.size.height,
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
    writer.write_image_data(&pixels)?;
    log::debug!("Converted to PNG in {:?}", pngstart.elapsed());
    anyhow::ensure!(
        !failure.load(Ordering::Relaxed),
        "Some pixels failed to render. The saved image may have gaps where this occurred."
    );
    Ok(())
}

pub(crate) fn do_save_state(path: &std::path::Path, state: UiState) -> anyhow::Result<()> {
    let data = UiStateSaveFile::from(state);
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &data)?;
    Ok(())
}
