//! A utility crate that builds the SPIR-V shader binary.
//!
//! Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{env, path::Path};

use spirv_builder::{SpirvBuilder, SpirvMetadata};
use thiserror::Error;

#[derive(Error, Debug)]
enum BuildError {
    #[error(transparent)]
    SpirvBuilder(#[from] spirv_builder::SpirvBuilderError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

fn build_shader(path_to_crate: &str) -> Result<(), BuildError> {
    build_print::info!("Building shader...");
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let features = if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        vec!["emulate_constants".into()]
    } else {
        vec![]
    };
    let mut builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1")
        .spirv_metadata(SpirvMetadata::None)
        .shader_crate_features(features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    builder.build_script.defaults = true;
    builder.build_script.env_shader_spv_path = Some(true);

    let compile_result = builder.build()?;
    // builder sets the env var brot3_lib.spv to the path of the built shader
    #[allow(clippy::disallowed_methods)]
    let shader_path = std::fs::canonicalize(compile_result.module.unwrap_single())?;
    build_print::info!("built shader is {shader_path:?}");
    Ok(())
}

fn main() -> Result<(), BuildError> {
    // CAUTION: Hard-wired path !
    build_shader("../lib")
}
