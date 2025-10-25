use std::env;
use std::path::{Path, PathBuf};

use cfg_aliases::cfg_aliases;

use std::fs::read_to_string;

/// Do we need to copy over a file?
///
/// Only if the destination file does not exist, or if its contents differ from the new version.
fn should_copy<P: AsRef<Path> + std::fmt::Debug, Q: AsRef<Path> + std::fmt::Debug>(
    new_file: P,
    destination: Q,
) -> std::io::Result<bool> {
    if !std::fs::exists(&destination)? {
        return Ok(true);
    }
    let data1 = read_to_string(new_file)?;
    let data2 = read_to_string(destination)?;
    Ok(data1 != data2)
}

/// Write a built file, but only put it into place if it differs from what's already there.
/// This prevents needless rebuilds based on timestamp comparison alone.
fn conditionally_write_built_file<P: AsRef<Path>>(cargo_manifest_dir: P) {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is required");
    let dst = Path::new(&out_dir).join("built.rs");
    let temp = Path::new(&out_dir).join("built.rs.new");
    built::write_built_file_with_opts(Some(cargo_manifest_dir.as_ref()), &temp)
        .expect("Failed to acquire build-time information");
    // Compare and move only if different
    if should_copy(&temp, &dst).unwrap() {
        std::fs::copy(temp, dst).unwrap();
    }
}

fn main() {
    let this_crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required");
    // CAUTION: Hard wired path
    println!("cargo:rerun-if-changed={this_crate_dir}/../.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
    conditionally_write_built_file(&this_crate_dir);

    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        we_compile: { all(
            any(feature = "hot-reload-shader", feature = "runtime-compilation"),
            not(wasm)
        )},
    }

    // We need a pre-compiled shader to use as a fallback.
    // Have we been provided with one? (CI artifact)
    if let Ok(shader_path) = env::var("BROT3_PREBUILT_SHADER") {
        // CAUTION: This must match what shader_builder main.rs outputs.
        println!("cargo::rustc-env=shader.spv={shader_path}");
    } else {
        // If not, go build it.
        build_shader();
    }
}

fn build_shader() {
    // CAUTION: Hard-wired paths !
    println!("cargo:rerun-if-changed=../shader_builder/");
    println!("cargo:rerun-if-changed=../shader/");
    println!("cargo:rerun-if-changed=../shader_common/");
    println!("cargo:rerun-if-changed=../shader_util/");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    // While OUT_DIR is set for both build.rs and compiling the crate, PROFILE is only set in
    // build.rs. So, export it to crate compilation as well.
    let profile = env::var("PROFILE").unwrap();
    println!("cargo:rustc-env=PROFILE={profile}");
    let mut dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // Strip `$profile/build/*/out`.
    let ok = dir.ends_with("out")
        && dir.pop()
        && dir.pop()
        && dir.ends_with("build")
        && dir.pop()
        && dir.ends_with(profile)
        && dir.pop();
    assert!(ok);
    // NOTE(eddyb) this needs to be distinct from the `--target-dir` value that
    // `spirv-builder` generates in a similar way from `$OUT_DIR` and `$PROFILE`,
    // otherwise repeated `cargo build`s will cause build script reruns and the
    // rebuilding of `rustc_codegen_spirv` (likely due to common proc macro deps).
    let dir = dir.join("builder");
    let status = std::process::Command::new("cargo")
        .args(["run", "--release", "-p", "shader_builder"])
        .arg("--target-dir")
        .arg(dir)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .status()
        .unwrap();
    // N.B. shader_builder outputs something like:
    // cargo::rustc-env=shader.spv=/home/builder/brot3/target/spirv-builder/spirv-unknown-vulkan1.1/release/deps/shader.spv
    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }
}
