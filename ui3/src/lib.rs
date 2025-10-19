#![allow(clippy::single_match)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod cli;
mod controller;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Options {}

const TITLE: &str = "brot3";

#[allow(dead_code)]
fn is_directory<P: AsRef<std::path::Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() -> anyhow::Result<()> {
    easy_shader_runner::setup_logging();

    let controller = controller::Controller::new(&Options {});
    cfg_if::cfg_if! {
        if #[cfg(all(
            any(feature = "hot-reload-shader", feature = "runtime-compilation"),
            not(target_arch = "wasm32")
        ))] {
            use clap::Parser;
            use std::path::PathBuf;

            // CAUTION: Hard-wired paths
            /// The relative path to the shader crate, from the ui3 crate
            const SHADER_RELATIVE_PATH: &str = "../shader";
            /// Where to look for the shader at runtime, if we're not running under cargo and no path was given
            const CANDIDATE_SHADER_PATHS: &[&str] = &["../shader", "./shader"];

            let args = cli::Args::parse();
            let manifest = std::env::var("CARGO_MANIFEST_DIR");
            let relative_to_manifest = manifest.is_ok();

            let mut shader_path = None;

            if let Ok(mp) = manifest {
                // We're running under cargo
                if args.shader.is_some() {
                    anyhow::bail!("Shader directory cannot be specified when CARGO_MANIFEST_DIR is set");
                }
                let mut pb = PathBuf::from(&mp);
                pb.push(SHADER_RELATIVE_PATH);
                if !is_directory(&pb) {
                    anyhow::bail!("Missing shader directory in CARGO_MANIFEST_DIR mode (manifest={mp}, shader={SHADER_RELATIVE_PATH})");
                }
                shader_path = Some(SHADER_RELATIVE_PATH.to_owned());
            } else {
                // We're not running under cargo
                if let Some(path) = args.shader.as_ref() {
                    if !is_directory(path) {
                        // If given, an explicit shader directory must be present
                        anyhow::bail!("Shader directory {path:?} not found");
                    }
                    shader_path = args.shader;
                } else {
                    for p in CANDIDATE_SHADER_PATHS {
                        if is_directory(p) {
                            shader_path = Some(p.to_string());
                            break;
                        }
                    }
                }
            }
            if let Some(path) = shader_path {
                easy_shader_runner::run_with_runtime_compilation_2(controller, path, TITLE, relative_to_manifest)?;
            } else {
                log::info!("Shader source directory not found, running with prebuilt shader");
                easy_shader_runner::run_with_prebuilt_shader_2(controller, include_bytes!(env!("shader.spv")), TITLE)?;
            }
        } else {
            // Runtime compilation disabled by feature flag
            easy_shader_runner::run_with_prebuilt_shader_2(controller, include_bytes!(env!("shader.spv")), TITLE)?;
        }
    }
    Ok(())
}
