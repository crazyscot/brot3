#![allow(clippy::single_match)]

#[cfg(wasm)]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod cli;
mod controller;

use clap::Parser;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const TITLE: &str = "brot3";

#[cfg(we_compile)]
fn is_directory<P: AsRef<std::path::Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

#[cfg(we_compile)]
fn is_file<P: AsRef<std::path::Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}

#[cfg_attr(wasm, wasm_bindgen(start))]
pub fn main() -> anyhow::Result<()> {
    easy_shader_runner::setup_logging();
    let args = cli::Args::parse();
    let controller = controller::Controller::new(&args);
    let params = easy_shader_runner::Parameters::new(controller, TITLE).esc_key_exits(false);
    cfg_if::cfg_if! {
        if #[cfg(we_compile)] {
            use std::path::PathBuf;

            // CAUTION: Hard-wired paths
            /// The relative path to the shader crate, from the point of view of the ui crate
            const CARGO_SHADER_RELATIVE_PATH: &str = "../shader";
            /// Where to look for the shader at runtime, if we're not running under cargo and no path was given
            const CANDIDATE_SHADER_PATHS: &[&str] = &["./shader", "../shader"];

            let manifest = std::env::var("CARGO_MANIFEST_DIR");
            let relative_to_manifest = manifest.is_ok();

            let mut shader_path = None;

            if let Ok(mp) = manifest {
                // We're running under cargo
                if args.shader.is_some() {
                    anyhow::bail!(
                        "Shader directory cannot be specified when CARGO_MANIFEST_DIR is set"
                    );
                }
                let mut pb = PathBuf::from(&mp);
                pb.push(CARGO_SHADER_RELATIVE_PATH);
                if !is_directory(&pb) {
                    anyhow::bail!("Missing shader directory in CARGO_MANIFEST_DIR mode (manifest={mp}, shader={CARGO_SHADER_RELATIVE_PATH})");
                }
                shader_path = Some(PathBuf::from(CARGO_SHADER_RELATIVE_PATH));
            } else {
                // We're not running under cargo
                if let Some(path) = args.shader.as_ref() {
                    if !is_directory(path) {
                        // If given, an explicit shader directory must be present
                        anyhow::bail!("Shader directory {path:?} not found");
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
                anyhow::bail!("SPIRV tools {tp:?} not found");
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
                easy_shader_runner::run_with_runtime_compilation_2(
                    params,
                    path,
                    relative_to_manifest,
                    args.spirv_tools,
                )?;
            } else {
                easy_shader_runner::run_with_prebuilt_shader_2(
                    params,
                    include_bytes!(env!("shader.spv")),
                )?;
            }
        } else {
            // Runtime compilation disabled by feature flag
            easy_shader_runner::run_with_prebuilt_shader_2(
                params,
                include_bytes!(env!("shader.spv")),
            )?;
        }
    }
    Ok(())
}
