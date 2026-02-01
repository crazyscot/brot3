#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{env, path::Path};

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn build_shader(path_to_crate: &str) -> anyhow::Result<()> {
    build_print::info!("Building shader...");
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let features = if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        vec!["emulate_constants".into()]
    } else {
        vec![]
    };
    let builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::None)
        .shader_crate_features(features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);

    let compile_result = builder.build()?;
    #[allow(clippy::disallowed_methods)]
    let shader_path = std::fs::canonicalize(compile_result.module.unwrap_single()).unwrap();
    let file_name = shader_path.file_name().unwrap().to_str().unwrap();
    // sample output:
    // cargo::rustc-env=shader.spv=/home/builder/brot3/target/spirv-builder/spirv-unknown-vulkan1.1/
    // release/deps/shader.spv CAUTION: This must match what ui/build.rs expects.
    println!("cargo::rustc-env={}={}", file_name, shader_path.display());
    build_print::info!("built shader is {shader_path:?}");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // CAUTION: Hard-wired path !
    build_shader("../shader")
}
