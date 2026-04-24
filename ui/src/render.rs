//! CLI rendering mode

use brot3_lib::ui::UiState;
use env_logger::Env;

use crate::{MainError, cli::Args, save::do_save_image};

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] brot3_lib::ui::Error),
    #[error("Failed to save output file: {0}")]
    SaveFailed(crate::save::LoadSaveError),
}

pub(crate) fn main(args: &Args) -> Result<(), MainError> {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("warn")).try_init();

    let input = args.input.as_ref().unwrap();
    let mut state = UiState::load_magic(input).map_err(RenderError::LoadFailed)?;

    state.viewport_size = args.size.unwrap_or_default().as_uvec2();
    let mut perturbation_points: Vec<glam::Vec2> = Vec::new();
    if state.viewport_zoom.requires_perturbation_mode() {
        brot3_lib::engine::mandelbrot_perturbed_compute_reference_iters(
            &mut perturbation_points,
            &state.viewport_translate,
            state.algorithm,
            state.max_iter,
        );
        // do_save_image will set the perturbation_mode flag
    }

    let output = args.output.as_ref().unwrap();
    log::debug!("Render mode is {}", args.render_mode);
    do_save_image(output, &state, &perturbation_points, args.render_mode)
        .map_err(RenderError::SaveFailed)?;
    log::info!("Successfully rendered image to {}", output.display());
    Ok(())
}
