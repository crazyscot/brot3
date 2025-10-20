use std::env;
use std::path::PathBuf;

use cfg_aliases::cfg_aliases;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    built::write_built_file().expect("Failed to acquire build-time information");

    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        we_compile: { all(
            any(feature = "hot-reload-shader", feature = "runtime-compilation"),
            not(wasm)
        )},
    }

    println!("cargo:rerun-if-changed=build.rs");
    // CAUTION: Hard-wired path !
    println!("cargo:rerun-if-changed=../shader_builder/");
    // CAUTION: Hard-wired path !
    println!("cargo:rerun-if-changed=../shader/");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    // Unconditionally build the shader so we have it around as a fallback.

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
    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }
}
