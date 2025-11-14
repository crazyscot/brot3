use std::env;
use std::path::PathBuf;

use cfg_aliases::cfg_aliases;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    process_version_string();

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

    // CAUTION: Hard wired path
    let top_level = {
        let mut temp =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required"));
        temp.pop();
        temp
    };
    println!(
        "cargo:rerun-if-changed={}/.git/HEAD",
        top_level.to_string_lossy()
    );

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
    use std::process::Command;
    if let Ok(output) = Command::new("git").args(args).output() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_short_hash() -> Option<String> {
    git_command(&["rev-parse", "--short=8", "HEAD"]).filter(|r| !r.is_empty())
}

fn git_is_dirty() -> Option<bool> {
    git_command(&["status", "--porcelain"]).map(|r| r.is_empty())
}

fn git_describe() -> Option<String> {
    git_command(&["describe", "--always", "--dirty"])
}
