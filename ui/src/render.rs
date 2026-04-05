//! CLI rendering mode

use brot3_lib::{
    data::{Flags, FragmentConstants},
    ui::UiState,
};

use crate::{MainError, cli::Args, save::do_save_image};

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] brot3_lib::ui::Error),
    #[error("Failed to save output file: {0}")]
    SaveFailed(crate::save::LoadSaveError),
}

pub(crate) fn main(args: &Args) -> Result<(), MainError> {
    let _ = env_logger::try_init();

    let input = args.input.as_ref().unwrap();
    let mut state = UiState::load_magic(input).map_err(RenderError::LoadFailed)?;

    state.viewport_size = args.size.unwrap_or_default().as_uvec2();
    let mut constants = FragmentConstants::from(&state);

    let mut perturbation_points = Vec::new();
    if f64::from(constants.viewport_zoom) > crate::MAX_ZOOM_STANDARD {
        constants.flags |= Flags::PERTURBATION_MODE;
        brot3_lib::engine::mandelbrot_perturbed_compute_reference_iters(
            &mut perturbation_points,
            &state.viewport_translate,
            state.algorithm,
            state.max_iter,
        );
    }

    let output = args.output.as_ref().unwrap();
    log::info!("Parallel flag is {}", !args.no_parallel_render);
    do_save_image(
        output,
        constants,
        &state,
        &perturbation_points,
        !args.no_parallel_render,
    )
    .map_err(RenderError::SaveFailed)?;
    log::info!("Successfully rendered image to {}", output.display());
    Ok(())
}
