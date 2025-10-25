use spirv_builder::SpirvBuilder;
use std::env;
use std::path::Path;

fn build_shader(path_to_crate: &str) -> anyhow::Result<()> {
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let mut builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1");
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        builder = builder.shader_crate_features(["emulate_constants".into()]);
    }
    let compile_result = builder.build()?;
    let shader_path = std::fs::canonicalize(compile_result.module.unwrap_single()).unwrap();
    let file_name = shader_path.file_name().unwrap().to_str().unwrap();
    // sample output: cargo::rustc-env=shader.spv=/home/builder/brot3/target/spirv-builder/spirv-unknown-vulkan1.1/release/deps/shader.spv
    // CAUTION: This must match what ui/build.rs expects.
    println!("cargo::rustc-env={}={}", file_name, shader_path.display());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // CAUTION: Hard-wired path !
    build_shader("../shader")
}
