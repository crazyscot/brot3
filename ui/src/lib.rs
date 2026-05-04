//! UI and main entrypoint for brot3

#![allow(clippy::single_match)]

#[cfg(wasm)]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod cli;
pub mod compute;
#[cfg(feature = "ui")]
mod controller;
mod render;
pub(crate) mod save;
pub(crate) mod version;
#[cfg(feature = "ui")]
pub mod widgets;

#[cfg(runtime_compile)]
use std::path::Path;

use clap::Parser;

// CAUTION: Hard-wired paths
/// The relative path to the shader crate, from the point of view of the ui crate
#[cfg(runtime_compile)]
const CARGO_SHADER_RELATIVE_PATH: &str = "../lib";
/// Where to look for the shader at runtime, if we're not running under cargo and no path was given
#[cfg(runtime_compile)]
const CANDIDATE_SHADER_PATHS: &[&str] = &["./lib", "../lib"];

#[doc(hidden)]
pub use save::write_png; // exported for use by compute_shader integration test
use version::version_string;

#[cfg(not(feature = "suppress-shader-build"))]
/// Single point of reference to the spir-v shader
const SHADER_BYTES: &[u8] = include_bytes!(env!("brot3_lib.spv"));

/// Absolute limit on the number of iterations, which also limits the size of the perturbation
/// buffer.
// N.B. This affects the perturbation buffer size. But it's only 2 * sizeof(f32) per point.
const MAX_MAX_ITERATIONS: usize = 100_000;

#[cfg(runtime_compile)]
fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

#[cfg(runtime_compile)]
fn is_file<P: AsRef<Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum MainError {
    #[error("Shader directory cannot be specified when CARGO_MANIFEST_DIR is set")]
    ManifestShaderConflict,
    #[error("Missing shader directory in CARGO_MANIFEST_DIR mode (manifest={mp}, shader={shader})")]
    MissingShaderDirectory { mp: String, shader: String },
    #[error("Shader directory {0} not found")]
    ShaderDirectoryNotFound(String),
    #[error("SPIRV tools {0} not found")]
    SpirvToolsNotFound(String),
    #[cfg(feature = "ui")]
    #[error(transparent)]
    EasyShaderRunner(#[from] easy_shader_runner::Error),
    #[error(transparent)]
    Render(#[from] render::RenderError),
    #[error("Shader not present due to suppress-shader-build feature")]
    SuppressedShaderBuild,
    #[error("User interface is not present in this build")]
    UilessBuild,
}

/// Main CLI entrypoint
#[tokio::main]
#[cfg_attr(wasm, wasm_bindgen(start))]
#[allow(clippy::missing_panics_doc)]
pub async fn main() -> Result<(), MainError> {
    let args = cli::Args::parse();
    if args.version {
        println!("{}", version_string("brot3 "));
        return Ok(());
    }

    if args.output.is_some() {
        render::main(&args)
    } else {
        cfg_if::cfg_if! {
            if #[cfg(feature = "ui")] {
                ui_main(&args)
            } else {
                Err(MainError::UilessBuild)
            }
        }
    }
}

#[cfg(feature = "ui")]
fn ui_main(args: &cli::Args) -> Result<(), MainError> {
    easy_shader_runner::setup_logging();
    let controller = controller::Controller::new(args);
    let params = easy_shader_runner::Parameters::new(controller, version_string("brot3 "))
        .esc_key_exits(false);
    cfg_if::cfg_if! {
        if #[cfg(feature = "suppress-shader-build")] {
            // Runtime compilation disabled by feature flag
            Err(MainError::SuppressedShaderBuild)?;
            let _ = params.esc_key_exits(true); // hush unused warning
        } else if #[cfg(runtime_compile)] {

            let manifest = std::env::var("CARGO_MANIFEST_DIR");
            let relative_to_manifest = manifest.is_ok();

            let mut shader_path = None;

            if let Ok(mp) = manifest {
                // We're running under cargo
                if args.shader.is_some() {
                    return Err(MainError::ManifestShaderConflict);
                }
                let mut pb = PathBuf::from(&mp);
                pb.push(CARGO_SHADER_RELATIVE_PATH);
                if !is_directory(&pb) {
                    return Err(MainError::MissingShaderDirectory {
                        mp,
                        shader: CARGO_SHADER_RELATIVE_PATH.to_string(),
                    });
                }
                shader_path = Some(PathBuf::from(CARGO_SHADER_RELATIVE_PATH));
            } else {
                // We're not running under cargo
                if let Some(path) = args.shader.as_ref() {
                    if !is_directory(path) {
                        // If given, an explicit shader directory must be present
                        return Err(MainError::ShaderDirectoryNotFound(path.display()));
                    }
                    shader_path = args.shader;
                } else if !args.static_shader {
                    for p in CANDIDATE_SHADER_PATHS {
                        if is_directory(p) {
                            shader_path = Some(PathBuf::from(p));
                            break;
                        }
                    }
                    if shader_path.is_none() {
                        log::info!(
                            "Shader source directory not found, running with prebuilt shader"
                        );
                    }
                }
            }
            if let Some(ref tp) = args.spirv_tools
                && !is_file(tp)
            {
                return Err(MainError::SpirvToolsNotFound(tp.display().to_string()));
            }
            if let Some(path) = shader_path
                && !args.static_shader
            {
                let hook = std::panic::take_hook();
                std::panic::set_hook(Box::new(move |e| {
                    let msg = e.to_string();
                    if msg.contains("Could not find") && msg.contains("in library path") {
                        eprintln!("Error: {e}\nEither set your library path appropriately, or specify the path to the library with --spirv-tools <PATH>, or use --static-shader");
                    } else {
                        hook(e);
                    }
                }));
                easy_shader_runner::run_with_runtime_compilation(
                    params,
                    path,
                    relative_to_manifest,
                    args.spirv_tools,
                )?;
            } else {
                easy_shader_runner::run_with_prebuilt_shader(
                    params,
                    SHADER_BYTES,
                )?;
            }
        } else {
            // Runtime compilation disabled by feature flag
            easy_shader_runner::run_with_prebuilt_shader(
                params,
                SHADER_BYTES,
            )?;
        }
    }
    Ok(())
}
