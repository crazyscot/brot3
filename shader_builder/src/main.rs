//! A utility crate that builds the SPIR-V shader binary.
//!
//! Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#![allow(missing_docs, clippy::missing_panics_doc)]

#[allow(dead_code, unreachable_pub)]
mod build_defs;

use std::{
    env,
    path::{Path, PathBuf},
};

use build_defs::ShaderVariant;
use spirv_builder::{SpirvBuilder, SpirvMetadata};
use strum::VariantArray as _;
use thiserror::Error;

#[derive(Error, Debug)]
enum BuildError {
    #[error(transparent)]
    SpirvBuilder(#[from] spirv_builder::SpirvBuilderError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

fn build_shader(path_to_crate: &str, variant: ShaderVariant) -> Result<PathBuf, BuildError> {
    build_print::info!("Building shader variant {}...", variant.key());
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let mut features = variant
        .shader_crate_features()
        .iter()
        .copied()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        features.push("emulate_constants".into());
    }
    let mut builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1")
        .spirv_metadata(SpirvMetadata::None)
        .shader_crate_features(features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    builder.build_script.defaults = true;
    builder.build_script.env_shader_spv_path = Some(false);

    #[allow(unsafe_code, clippy::disallowed_methods)]
    unsafe {
        std::env::set_var("CARGO_PROFILE_RELEASE_DEBUG", "false");
    }

    let compile_result = builder.build()?;
    // spirv-builder reuses a single output path, so copy each finished build to a stable
    // variant-specific path before the next variant overwrites it.
    #[allow(clippy::disallowed_methods)]
    let shader_path = dunce::canonicalize(compile_result.module.unwrap_single())?;
    let final_path = shader_path.with_file_name(variant.shader_filename());
    let _bytes_copied = std::fs::copy(&shader_path, &final_path)?;
    let final_path = dunce::canonicalize(final_path)?;
    build_print::info!("built shader variant {} at {:?}", variant.key(), final_path);
    Ok(final_path)
}

fn main() -> Result<(), BuildError> {
    // CAUTION: Hard-wired path !
    let mut prebuild_shaders_dir = None;
    for variant in ShaderVariant::VARIANTS {
        let built_shader = build_shader("../lib", *variant)?;
        let built_shader_dir = built_shader.parent().unwrap().to_path_buf();
        if let Some(existing_dir) = &prebuild_shaders_dir {
            assert_eq!(existing_dir, &built_shader_dir);
        } else {
            prebuild_shaders_dir = Some(built_shader_dir);
        }
    }
    let prebuild_shaders_dir = prebuild_shaders_dir.unwrap();
    println!(
        "cargo:rustc-env={}={}",
        ShaderVariant::prebuild_shaders_dir_env_var(),
        prebuild_shaders_dir.display()
    );
    Ok(())
}
