//! xtask to update the changelog
// (c) 2025 Ross Younger

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context as _, Result};
use pico_args::Arguments;

use crate::util::{ensure_all_args_used, top_level};

pub fn changelog(mut cli_args: Arguments) -> Result<()> {
    let bump: Option<String> = cli_args.opt_value_from_str("--bump")?;
    ensure_all_args_used(cli_args)?;
    // Path wrangling
    let mut path: PathBuf = top_level()?;
    path.push("CHANGELOG.md");
    let orig = path.clone();
    let _ = path.pop();
    path.push("CHANGELOG.md.new");
    let new = path;

    // git cliff --unreleased [--bump <bump>] & capture the output
    let mut cliff = Command::new("git");
    let _ = cliff.args(["cliff", "--unreleased"]);
    if let Some(bbump) = bump {
        let _ = cliff.args(["--bump", &bbump]);
    }
    let output = cliff.output().context("invoking git cliff")?;
    anyhow::ensure!(output.status.success(), "git cliff did not succeed");
    let mut output = String::from(str::from_utf8(output.stdout.trim_ascii())?);

    // prepend to the existing changelog
    let contents = std::fs::read_to_string(&orig).context("reading changelog")?;
    output.push_str("\n\n");
    output.push_str(&contents);
    // write to new file
    std::fs::write(&new, output).context("writing new changelog")?;
    // move new file over old
    let _ = std::fs::copy(&new, orig)?;
    std::fs::remove_file(new)?;

    Ok(())
}
