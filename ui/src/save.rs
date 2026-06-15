//! Save and Load support
// (c) 2026 Ross Younger

use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant},
};

use brot3_lib::{
    data::{Flags, FragmentConstants, RenderMode},
    ui::{Error as LibError, ShaderVariant, UiState},
    util::render_frame,
};
use easy_cast::CastApprox;
use glam::{Vec2, uvec2};
use thiserror::Error;

use crate::compute::ComputeController;

/// The primary error type used by this module
#[derive(Error, Debug, strum::EnumIs)]
#[allow(missing_docs)]
pub enum LoadSaveError {
    #[error(transparent)]
    Internal(#[from] LoadSaveErrorInternal),
    #[error("Compute shader controller failed: {0}")]
    ComputeController(#[from] crate::compute::ComputeControllerError),
}

/// Internal errors related to saving files. (The hierarchy is necessary to avoid type recursion.)
#[derive(Error, Debug, strum::EnumIs)]
#[allow(missing_docs)]
pub enum LoadSaveErrorInternal {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG encoding error: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[error("{0}")]
    Lib(#[from] LibError),
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
    match mode {
        RenderMode::Cpu | RenderMode::CpuParallel => {
            let parallel = matches!(mode, RenderMode::CpuParallel);
            let pixels = render_frame(&constants, perturbation_points, parallel);
            write_png(path, state, &pixels)?;
        }
        RenderMode::Gpu => {
            render_gpu(state, &constants, perturbation_points, path)?;
        }
    }
    let duration = start.elapsed();
    log::debug!("Overall image save took {duration:?}");
    Ok(())
}

fn render_gpu(
    state: &UiState,
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
    path: &Path,
) -> Result<(), LoadSaveError> {
    use easy_cast::Conv as _;
    let render_size = constants.size.into();
    let mut controller =
        ComputeController::new(render_size, 1, ShaderVariant::from_ui_state(state))?;
    let times = controller.run(
        *constants,
        render_size.extend(1),
        1,
        perturbation_points,
        |rgba| write_png(path, state, rgba),
    )?;
    if times.len() == 4 {
        use itertools::Itertools as _;
        // The compute controller provides 4 timestamps: start, start of compute, completion of
        // compute, teardown. These in turn can be resolved into phases: setup, compute,
        // teardown.
        let resolution = controller.get_timestamp_period();
        let overall = Duration::from_nanos(
            (f64::conv(times.last().unwrap() - times[0]) * resolution).cast_approx(),
        );
        let deltas = times
            .into_iter()
            .tuple_windows()
            .map(|(start, end)| {
                Duration::from_nanos((f64::conv(end - start) * resolution).cast_approx())
            })
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
    Ok(())
}

/// Writes the given pixel data to a PNG file, embedding metadata about the UI state and
/// software version.
pub fn write_png(path: &Path, state: &UiState, pixels: &[u8]) -> Result<(), LoadSaveErrorInternal> {
    let pngstart = Instant::now();
    let file = File::create(path)?;
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
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
    log::debug!("Wrote PNG in {:?}", pngstart.elapsed());
    Ok(())
}
