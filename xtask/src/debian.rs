//! xtask to create debian package files
// (c) 2025 Ross Younger

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use cargo_toml::Manifest;
use pico_args::Arguments;

use crate::util::{ensure_all_args_used, gzip, top_level};

static DEBEMAIL: &str = "ross@crazyscot.com";
static DEBFULLNAME: &str = "Ross Younger";
static DISTRO: &str = "generic";

#[derive(derive_more::Constructor)]
pub struct DebPackageMeta {
    pub deb_name: &'static str,
    pub package_crate: &'static str,
    pub package_dir: &'static str,
    pub deb_fullname: &'static str,
    pub deb_email: &'static str,
    pub distro: &'static str,
}

pub fn debian(mut cli_args: Arguments, args: &DebPackageMeta) -> Result<()> {
    let revision = if cli_args.contains("--release") {
        String::new()
    } else {
        crate::util::git_short_hash()?
    };
    let no_build = cli_args.contains("--no-build");
    let target: Option<String> = cli_args.opt_value_from_str("--target")?;
    ensure_all_args_used(cli_args)?;
    let toplevel: PathBuf = top_level()?;
    let mut target_misc = toplevel.clone();
    target_misc.push("target");
    target_misc.push("misc");
    std::fs::create_dir_all(&target_misc)?;

    // Prepare package changelog
    gzip(
        toplevel.clone().join("CHANGELOG.md"),
        target_misc.clone().join("changelog.md.gz"),
    )?;

    // Prepare dummy Debian changelog
    create_dch(args, &target_misc)?;
    gzip(
        target_misc.clone().join("changelog.Debian"),
        target_misc.clone().join("changelog.Debian.gz"),
    )?;

    println!("Running cargo deb...");
    let mut cargo_deb = Command::new("cargo");
    let _ = cargo_deb
        .args([
            "deb",
            "-p",
            args.package_crate,
            "--locked",
            "--profile",
            "release",
            "--deb-revision",
            &revision,
        ])
        .stderr(Stdio::inherit());
    if no_build {
        let _ = cargo_deb.args(["--no-build"]);
    }
    if let Some(tgt) = target {
        let _ = cargo_deb.args(["--target", &tgt]);
    }
    let cargo_deb = cargo_deb.output().context("Invoking cargo deb")?;
    if !cargo_deb.status.success() {
        anyhow::bail!("cargo deb did not succeed");
    }
    let result = str::from_utf8(cargo_deb.stdout.trim_ascii())?;
    // Strip any warnings; the deb filename is the last "word" printed
    let Some(debfile) = result.split_ascii_whitespace().last() else {
        anyhow::bail!("cargo deb did not output a debfile");
    };
    if let Ok(gh_outfile) = std::env::var("GITHUB_OUTPUT") {
        let debout = format!("deb={debfile}\n");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&gh_outfile)?;
        file.write_all(debout.as_bytes())?;
        file.flush()?;
        drop(file);
        println!("Wrote {} to GITHUB_OUTPUT", debout.trim_ascii_end());
    } else {
        println!("Created deb: {debfile}");
    }
    Ok(())
}

/// Create dummy Debian changelog in destdir/changelog.Debian
fn create_dch(args: &DebPackageMeta, destdir: &Path) -> Result<()> {
    // Get package cargo version
    let path = PathBuf::from(&args.package_dir).join("Cargo.toml");
    let m: Manifest = Manifest::from_path(path)?;
    let version = m.package().version();

    // Traditionally the developer would invoke the `dch' script (in devscripts).
    // But we know exactly what this file has to contain, so we'll cut to the chase.
    let outpath = destdir.to_path_buf().join("changelog.Debian");
    let mut outfile = File::create(&outpath)?;
    let date = chrono::Local::now().to_rfc2822();
    write!(
        outfile,
        r"{package} ({version}) {DISTRO}; urgency=medium

  * New upstream release.
    See /usr/share/doc/{package}/changelog.gz for full details.

 -- {DEBFULLNAME} <{DEBEMAIL}>  {date}
",
        package = args.deb_name,
    )?;
    outfile.flush()?;
    println!("Wrote changelog to {}", outpath.display());
    Ok(())
}
