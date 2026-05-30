//! UI and main entrypoint for brot3

#![allow(clippy::single_match)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod cli;
pub mod compute;
#[cfg(feature = "ui")]
mod controller;
mod render;
pub(crate) mod save;
mod shader_assets;
pub(crate) mod version;
#[cfg(feature = "ui")]
pub mod widgets;

#[cfg(feature = "hot-reload-shader")]
use std::path::{Path, PathBuf};

#[cfg(any(feature = "hot-reload-shader", feature = "ui"))]
use brot3_lib::ui::ShaderVariant;
use clap::Parser;

// CAUTION: Hard-wired paths
/// The relative path to the shader crate, from the point of view of the ui crate
#[cfg(feature = "hot-reload-shader")]
const CARGO_SHADER_RELATIVE_PATH: &str = "../lib";
/// Where to look for the shader at runtime, if we're not running under cargo and no path was given
#[cfg(feature = "hot-reload-shader")]
const CANDIDATE_SHADER_PATHS: &[&str] = &["./lib", "../lib"];

#[doc(hidden)]
pub use save::write_png; // exported for use by compute_shader integration test
use version::version_string;

pub(crate) const SHADER_BYTES: &[u8] = shader_assets::prebuilt_shader_bytes(ShaderVariant::General);

#[cfg(feature = "ui")]
const PREBUILT_SHADERS: &[easy_shader_runner::PrebuiltShader] = &[
    easy_shader_runner::PrebuiltShader::new(
        ShaderVariant::General.key(),
        shader_assets::prebuilt_shader_bytes(ShaderVariant::General),
    ),
    easy_shader_runner::PrebuiltShader::new(
        ShaderVariant::MandelbrotPow2.key(),
        shader_assets::prebuilt_shader_bytes(ShaderVariant::MandelbrotPow2),
    ),
    easy_shader_runner::PrebuiltShader::new(
        ShaderVariant::MandelbrotPow2Deep.key(),
        shader_assets::prebuilt_shader_bytes(ShaderVariant::MandelbrotPow2Deep),
    ),
];

#[cfg(feature = "hot-reload-shader")]
const RUNTIME_GENERAL_SHADER: easy_shader_runner::RuntimeCompilationShader =
    easy_shader_runner::RuntimeCompilationShader::new(
        ShaderVariant::General.key(),
        ShaderVariant::General.shader_crate_features(),
    );

#[cfg(false)] // TEMP for now as not yet used
#[cfg(feature = "hot-reload-shader")]
const RUNTIME_SHADERS: &[easy_shader_runner::RuntimeCompilationShader] = &[
    RUNTIME_GENERAL_SHADER,
    easy_shader_runner::RuntimeCompilationShader::new(
        ShaderVariant::MandelbrotPow2.key(),
        ShaderVariant::MandelbrotPow2.shader_crate_features(),
    ),
    easy_shader_runner::RuntimeCompilationShader::new(
        ShaderVariant::MandelbrotPow2Deep.key(),
        ShaderVariant::MandelbrotPow2Deep.shader_crate_features(),
    ),
];

/// Absolute limit on the number of iterations, which also limits the size of the perturbation
/// buffer.
// N.B. This affects the perturbation buffer size. But it's only 2 * sizeof(f32) per point.
const MAX_MAX_ITERATIONS: usize = 100_000;

#[cfg(feature = "hot-reload-shader")]
fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

#[cfg(feature = "hot-reload-shader")]
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

    if args.output.is_some() {
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
        .default_shader_key(ShaderVariant::General.key());

    #[cfg(feature = "hot-reload-shader")]
    if !args.static_shader {
        match find_shader_path(args) {
            Ok(path) => {
                log::info!(
                    "Found shader source directory at {}, running with runtime compilation",
                    path.display()
                );
                return run_with_hot_reload(args, &path, params);
            }
            Err(MainError::FallbackToPrebuiltShader) => {
                log::warn!("Shader source directory not found, running with prebuilt shaders");
            }
            Err(e) => return Err(e),
        }
    }
    // runtime compilation configured out, or it didn't succeed
    Ok(easy_shader_runner::run_with_prebuilt_shaders(
        params,
        PREBUILT_SHADERS,
    )?)
}

#[cfg(feature = "hot-reload-shader")]
/// Finds the path to the shader source directory, if it exists.
///
/// Returns `Ok(path to shader)` on success.
fn find_shader_path(args: &cli::Args) -> Result<PathBuf, MainError> {
    let shader_path = if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        // We're running under cargo
        if args.shader.is_some() {
            return Err(MainError::ManifestShaderConflict);
        }
        let mut pb = PathBuf::from(&manifest);
        pb.push(CARGO_SHADER_RELATIVE_PATH);
        if !is_directory(&pb) {
            return Err(MainError::MissingShaderDirectory {
                mp: manifest,
                shader: CARGO_SHADER_RELATIVE_PATH.to_string(),
            });
        }
        Some(PathBuf::from(CARGO_SHADER_RELATIVE_PATH))
    } else {
        // We're not running under cargo
        if let Some(path) = args.shader.as_ref() {
            if !is_directory(path) {
                // If given, an explicit shader directory must be present
                return Err(MainError::ShaderDirectoryNotFound);
            }
            args.shader.clone()
        } else {
            CANDIDATE_SHADER_PATHS
                .iter()
                .find(|p| is_directory(p))
                .map(PathBuf::from)
        }
    };
    if let Some(ref tp) = args.spirv_tools
        && !is_file(tp)
    {
        return Err(MainError::SpirvToolsNotFound(tp.display().to_string()));
    }
    shader_path.ok_or(MainError::FallbackToPrebuiltShader)
}

#[cfg(feature = "hot-reload-shader")]
fn run_with_hot_reload<C: easy_shader_runner::ControllerTrait + Send>(
    args: &cli::Args,
    path: &Path,
    params: easy_shader_runner::Parameters<C>,
) -> Result<(), MainError> {
    // Are we running under cargo?
    let relative_to_manifest = std::env::var("CARGO_MANIFEST_DIR").is_ok();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        let msg = e.to_string();
        if msg.contains("Could not find") && msg.contains("in library path") {
            eprintln!(
                "Error: {e}\nEither set your library path appropriately, or specify the path to the library with --spirv-tools <PATH>, or use --static-shader"
            );
        } else {
            hook(e);
        }
    }));

    easy_shader_runner::run_with_runtime_compilation(
        params,
        &[RUNTIME_GENERAL_SHADER], /* TODO: Ensure that hot reload selects the correct variant.
                                    * RUNTIME_SHADERS comes into play. */
        path,
        relative_to_manifest,
        args.spirv_tools.as_ref(),
    )?;
    Ok(())
}
