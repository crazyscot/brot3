//! Save and Load support
// (c) 2026 Ross Younger

use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use brot3_lib::{
    data::{Flags, FragmentConstants, PointResult},
    ui::{Error as LibError, UiState, UiStateSaveFile},
};
use glam::{Vec2, Vec4, uvec2, vec4};
use rayon::prelude::*;
use thiserror::Error;

/// The error type used by this module
#[derive(Error, Debug, strum::EnumIs)]
pub enum LoadSaveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG encoding error: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[error("PNG decoding error: {0}")]
    PngDecode(#[from] png::DecodingError),
    #[error("Unrecognised file format")]
    UnrecognisedFormat,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Lib(#[from] LibError),
    #[error("Some pixels failed to render. The saved image may have gaps where this occurred.")]
    PartialRenderFailure,
    #[error("PNG file did not contain usable state data")]
    PngHadNoStateData,
}

pub(crate) fn do_save_image(
    path: &Path,
    mut constants: FragmentConstants,
    state: &UiState,
    perturbation_points: &[Vec2],
    parallel: bool,
) -> Result<(), LoadSaveError> {
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

    let render_chunk = |(chunk_idx, chunk): (usize, &mut [u8])| {
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
    if failure.load(Ordering::Relaxed) {
        return Err(LoadSaveError::PartialRenderFailure);
    }
    Ok(())
}

pub(crate) fn do_save_state(path: &Path, state: UiState) -> Result<(), LoadSaveError> {
    let data = UiStateSaveFile::from(state);
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &data)?;
    log::info!("Saved state to {}", path.display());
    Ok(())
}

/// Loads the state from the given file.
///
/// *NOTE:* Caller is responsible for figuring out whether to enable perturbation mode or other
/// flags based on the new state.
pub(crate) fn load_state(path: &Path) -> Result<UiState, LoadSaveError> {
    // Some errors are fatal (e.g. file not found), but if the file is there and it's just not valid
    // JSON, we want to try loading as a PNG before giving up.
    match load_state_json(path) {
        Ok(state) => {
            log::info!("Loaded from {}", path.display());
            Ok(state)
        }
        Err(LoadSaveError::Json(j)) => {
            if j.is_syntax() {
                // It's not valid JSON, so try PNG
                load_state_png(path)
            } else {
                // I/O error, valid JSON that couldn't deserialize, premature EOF: all fatal
                Err(LoadSaveError::Json(j))
            }
        }
        Err(e) => Err(e),
    }
}

fn load_state_json(path: &Path) -> Result<UiState, LoadSaveError> {
    let file = File::open(path)?;
    let data: UiStateSaveFile = serde_json::from_reader(file)?;
    data.try_into().map_err(Into::into)
}

fn load_state_png(path: &Path) -> Result<UiState, LoadSaveError> {
    let decoder = png::Decoder::new(BufReader::new(File::open(path)?));
    let reader = decoder.read_info().map_err(|e| {
        if let png::DecodingError::Format(_) = e {
            // It's probably not a PNG at all
            LoadSaveError::UnrecognisedFormat
        } else {
            LoadSaveError::PngDecode(e)
        }
    })?;
    for chunk in &reader.info().uncompressed_latin1_text {
        if chunk.keyword == "uistate" {
            // Woo-hoo! It's for us!
            return serde_json::from_str(&chunk.text).map_err(Into::into);
        }
    }
    Err(LoadSaveError::PngHadNoStateData)
}
