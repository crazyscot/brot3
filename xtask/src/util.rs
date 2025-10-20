//! General utilities for xtasks
// (c) 2025 Ross Younger

#![allow(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]

use anyhow::{Context, Result};
use pico_args::Arguments;

use std::path::PathBuf;
use std::process::Command;

pub type TaskDefinition<'a> = (&'a str, fn(Arguments) -> Result<()>, &'a str);
pub type Tasks<'a> = &'a [TaskDefinition<'a>];

pub fn dispatch(tasks: Tasks<'_>) -> Result<()> {
    ensure_top_level()?;
    let mut args = Arguments::from_env();
    let cmd = args.subcommand()?;
    if let Some(task) = cmd.as_deref() {
        tasks
            .iter()
            .find_map(|(verb, fun, _)| (*verb == task).then_some(*fun))
            .map(|f| f(args))
            .or_else(|| Some(help(tasks)))
            .unwrap_or_else(|| anyhow::bail!("logic error"))
    } else {
        help(tasks)
    }?;
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn help(tasks: Tasks<'_>) -> Result<()> {
    println!("Supported tasks:");
    let longest = tasks
        .iter()
        .fold(0, |acc, (verb, _, _)| std::cmp::max(acc, verb.len()));
    let mut display: Vec<_> = tasks.iter().collect();
    display.sort_by_key(|(verb, _, _)| *verb);
    for (verb, _, msg) in display {
        println!("  {verb:longest$}  {msg}");
    }
    Ok(())
}

pub fn top_level() -> Result<PathBuf> {
    let git_revparse = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Invoking git rev-parse")?;
    if !git_revparse.status.success() {
        anyhow::bail!("Failed to invoke git rev-parse");
    }
    Ok(PathBuf::from(str::from_utf8(
        git_revparse.stdout.trim_ascii(),
    )?))
}

pub fn ensure_top_level() -> Result<()> {
    std::env::set_current_dir(top_level()?).context("Changing to toplevel")?;
    Ok(())
}

pub fn ensure_all_args_used(args: Arguments) -> Result<()> {
    let unused = args.finish();
    anyhow::ensure!(
        unused.is_empty(),
        format!("Unhandled arguments: {unused:?}"),
    );
    Ok(())
}

/// This is essentially the `gzip` shell command
pub fn gzip(from: PathBuf, to: PathBuf) -> Result<()> {
    use flate2::{Compression, GzBuilder};
    use std::fs::File;
    use std::io::BufReader;

    let filename = from.clone();
    let filename = filename
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("failed to determine filename"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("failed to determine filename"))?;
    let infile = File::open(from)?;
    let mut inbuffer = BufReader::new(infile);
    let outfile = File::create(to)?;
    let mut gz = GzBuilder::new()
        .filename(filename)
        .write(outfile, Compression::default());
    let _ = std::io::copy(&mut inbuffer, &mut gz)?;
    let _ = gz.finish()?;
    Ok(())
}

pub fn git_short_hash() -> Result<String> {
    let git_revparse = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .context("Invoking git rev-parse")?;
    if !git_revparse.status.success() {
        anyhow::bail!("Failed to invoke git rev-parse");
    }
    Ok(str::from_utf8(git_revparse.stdout.trim_ascii())?.into())
}
