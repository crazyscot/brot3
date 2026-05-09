//! Build script for the UI crate, responsible for building shaders and computing version
//! information.
//!
//! Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{env, path::PathBuf};

#[cfg(all(target_arch = "wasm32", feature = "hot-reload-shader"))]
compile_error!("The `hot-reload-shader` feature is not supported on wasm builds.");

fn main() {
    process_version_string();

    // We need a pre-compiled shader to use as a fallback.
    // Have we been provided with one? (CI artifact)
    println!("cargo:rerun-if-env-changed=BROT3_PREBUILT_SHADER");
    if let Ok(shader_path) = env::var("BROT3_PREBUILT_SHADER") {
        // CAUTION: This must match what shader_builder main.rs outputs.
        build_print::info!("Using prebuilt shader at {shader_path}");
        println!("cargo:rustc-env=brot3_lib.spv={shader_path}");
    } else if cfg!(feature = "suppress-shader-build") {
        build_print::note!("Suppressing shader build due to feature flag");
    } else {
        // If not, go build it.
        build_print::note!("Running shader builder...");
        build_shader();
    }
}

fn build_shader() {
    // These days, spirv-builder outputs a lot of cargo:rerun-if-changed markers, so we don't need
    // to worry.

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
    #[allow(clippy::disallowed_methods)]
    let mut cargo = std::process::Command::new("cargo");
    #[allow(unused_results)]
    cargo
        .args(["run", "--release", "-p", "shader_builder", "-v"])
        .arg("--target-dir")
        .arg(dir)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit());
    let argz = cargo.get_args().collect::<Vec<_>>();
    build_print::info!("running: cargo {argz:?}");
    let status = cargo.status().unwrap();
    // N.B. shader_builder outputs something like:
    // `cargo:rustc-env=brot3_lib.spv=/home/builder/brot3/target/spirv-builder/
    // spirv-unknown-vulkan1.1/release/deps/brot3_lib.spv`
    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }
}

fn process_version_string() {
    /*
       Version string cases:

       Untagged CI build (SHORT_HASH present and == GIT_VERSION)
           => "PKG_VERSION-SHORT_HASH"
       Tagged CI build (GIT_VERSION present and != SHORT_HASH)
           => GIT_VERSION
       Non-CI build
           => use git describe output aka GIT_VERSION
       fallback (no git information) => PKG_VERSION-unknown

       In all cases, check for a dirty build and include that marker.

       In all cases, the version string must not contain any spaces (it's used by CI).
    */

    let pkgver = env!("CARGO_PKG_VERSION");
    // trap: docs.rs builds don't get a git short hash
    let short_hash = git_short_hash().unwrap_or("unknown".into());

    let dirty = match git_is_dirty() {
        Some(true) => "-dirty",
        Some(false) | None => "",
    };

    let running_in_ci = option_env!("CI").is_some();
    let ver = if running_in_ci {
        // CI builds generally have shallow clones, so git describe doesn't work as intended.
        if let Some(tag) = github_tag() {
            // This is a tagged CI build
            format!("{tag}{dirty}")
        } else {
            // This is an untagged CI build
            format!("{pkgver}-{short_hash}{dirty}")
        }
    } else if let Some(desc) = git_describe() {
        // Normal desktop development case
        format!("{desc}{dirty}")
    } else {
        // Fallback case (git didn't work?!)
        format!("{pkgver}-unknown{dirty}")
    };
    assert!(
        !ver.contains(' '),
        "the computed version string was not supposed to contain spaces"
    );

    println!("cargo:rustc-env=BROT3_VERSION_STRING={ver}");
    // access the result via env! or option_env!
    if running_in_ci {
        println!("cargo:warning=This is version {ver}");
    }

    // Force a rerun on change of branch or on commit
    // CAUTION: Hard wired path
    // TRAP: In the past, you could not use an absolute path with cargo:rerun-if-changed.
    // TRAP: Don't pretty-print a PathBuf here, you get quotes with it: cargo doesn't dequote, so it
    // will be always-dirty.
    let top_level = PathBuf::from("..");
    let index = top_level.clone().join(".git").join("index");
    println!("cargo:rerun-if-changed={}", index.display());
    let head = top_level.clone().join(".git").join("HEAD");
    println!("cargo:rerun-if-changed={}", head.display());

    if running_in_ci {
        // Put the string somewhere CI can read it.. CAUTION: Hard wired path
        let outfile = top_level
            .clone()
            .join("target")
            .join(std::env::var("PROFILE").expect("PROFILE is required"))
            .join("brot3.build-version.txt");
        std::fs::write(outfile, ver).unwrap();
    }
}

fn github_tag() -> Option<String> {
    std::env::var("GITHUB_REF_TYPE")
        .is_ok_and(|v| v == "tag")
        .then(|| std::env::var("GITHUB_REF_NAME").unwrap())
}

fn git_command(args: &[&str]) -> Option<String> {
    #[allow(clippy::disallowed_methods)]
    if let Ok(output) = std::process::Command::new("git").args(args).output() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_short_hash() -> Option<String> {
    git_command(&["rev-parse", "--short=8", "HEAD"]).filter(|r| !r.is_empty())
}

fn git_is_dirty() -> Option<bool> {
    git_command(&["status", "--porcelain"]).map(|r| !r.is_empty())
}

fn git_describe() -> Option<String> {
    git_command(&["describe", "--always"])
}
