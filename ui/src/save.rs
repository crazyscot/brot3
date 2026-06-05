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
        RenderMode::Cpu => render_frame(&constants, perturbation_points, false),
        RenderMode::CpuParallel => render_frame(&constants, perturbation_points, true),
        RenderMode::Gpu => render_gpu(state, &constants, perturbation_points)
            .inspect_err(|e| log::warn!("Failed to render on GPU, falling back to CPU: {e}"))
            .map_or_else(
                |_| render_frame(&constants, perturbation_points, true),
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

fn render_gpu(
    state: &UiState,
    constants: &FragmentConstants,
    perturbation_points: &[Vec2],
) -> Result<Vec<u8>, LoadSaveError> {
    use easy_cast::{Cast as _, Conv as _};
    let render_size = constants.size.into();
    let mut controller =
        ComputeController::new(render_size, 1, ShaderVariant::from_ui_state(state))?;
    let total_bytes = constants.size.element_product() * 4;
    let mut frame_data = Vec::with_capacity(total_bytes.cast());
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
        let resolution = controller.get_timestamp_period();
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
    Ok(frame_data)
}

/// Writes the given pixel data to a PNG file, embedding metadata about the UI state and
/// software version.
pub fn write_png(path: &Path, state: &UiState, pixels: &[u8]) -> Result<(), LoadSaveError> {
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
    Ok(())
}
