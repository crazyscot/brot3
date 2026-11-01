//! UI and main entrypoint for brot3

#![allow(clippy::single_match)]

use brot3_lib::{
    ui::{ShaderKey, UiState},
    wgsl,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod cli;
pub mod compute;
#[cfg(feature = "ui")]
mod controller;
mod render;
pub(crate) mod save;
mod shader;
pub(crate) mod version;
#[cfg(feature = "ui")]
pub mod widgets;

use clap::Parser;
#[doc(hidden)]
pub use save::write_png; // exported for use by compute_shader integration test
use version::version_string;

/// Absolute limit on the number of iterations, which also limits the size of the perturbation
/// buffer.
// N.B. This affects the perturbation buffer size. But it's only 2 * sizeof(f32) per point.
const MAX_MAX_ITERATIONS: usize = 100_000;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum MainError {
    #[error("Shader directory cannot be specified when CARGO_MANIFEST_DIR is set")]
    ManifestShaderConflict,
    #[error("Missing shader directory in CARGO_MANIFEST_DIR mode (manifest={mp}, shader={shader})")]
    MissingShaderDirectory { mp: String, shader: String },
    #[error("Shader directory not found")]
    ShaderDirectoryNotFound,
    #[error("SPIRV tools {0} not found")]
    SpirvToolsNotFound(String),
    #[cfg(feature = "ui")]
    #[error(transparent)]
    EasyShaderRunner(#[from] easy_shader_runner::Error),
    #[error(transparent)]
    Render(#[from] render::RenderError),
    #[error("User interface is not present in this build")]
    UilessBuild,
    #[error("Shader source directory not found, running with prebuilt shader")]
    FallbackToPrebuiltShader,
    #[error("WGSL debug output failed")]
    WgslDebugOutputFailed(String),
}

/// Main CLI entrypoint
#[tokio::main]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[allow(clippy::missing_panics_doc)]
pub async fn main() -> Result<(), MainError> {
    let args = cli::Args::parse();
    if args.version {
        println!("{}", version_string("brot3 "));
        return Ok(());
    }

    if args.debug_wgsl {
        let state = UiState::from(&args);
        let shader = wgsl::debug_output(ShaderKey::from(&state), args.debug_compute);
        if let Some(output) = &args.output {
            std::fs::write(output, &shader)
                .map_err(|_| MainError::WgslDebugOutputFailed("Failed to write output".into()))?;
        } else {
            println!("{shader}");
        }
        Ok(())
    } else if args.output.is_some() {
        render::main(&args)
    } else {
        #[cfg(feature = "ui")]
        {
            ui_main(&args)
        }
        #[cfg(not(feature = "ui"))]
        Err(MainError::UilessBuild)
    }
}

#[cfg(feature = "ui")]
fn ui_main(args: &cli::Args) -> Result<(), MainError> {
    easy_shader_runner::setup_logging();
    let controller = controller::Controller::new(args);
    #[allow(unused_variables, reason = "false positive")]
    let params = easy_shader_runner::Parameters::new(controller, version_string("brot3 "))
        .esc_key_exits(false)
        .default_shader_key("Fragment_MandelbrotPow2");

    Ok(easy_shader_runner::run_with_wgsl_shaders(params, &[])?)
}
