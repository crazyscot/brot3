// Show subcommand
// (c) 2024 Ross Younger

use brot3_engine::util::build_info;

#[derive(Debug, clap::Subcommand)]
enum ShowableThings {
    /// Build/Compiler information
    #[clap(alias = "b")]
    Build,
    /// License information
    #[clap(alias = "l")]
    License,
    /// Detailed version information
    #[clap(alias = "v")]
    Version,
}

/// Arguments to 'show'
#[derive(Debug, clap::Args)]
//#[command(flatten_help = true)]
pub(crate) struct Args {
    #[arg(
        long, hide(true), action = clap::ArgAction::Help, required(false)
    )]
    help: Option<bool>,

    #[command(subcommand)]
    thing: ShowableThings,
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn show(args: &Args) -> anyhow::Result<()> {
    let dirty_flag: String = match build_info::GIT_DIRTY {
        Some(x) => x.to_string(),
        None => "unknown".to_string(),
    };
    match args.thing {
        ShowableThings::Version => println!(
            "This is `{}' version {}.\n\
             Its git hash is: {}\n\
             The autogenerated git version is {} and the dirty flag is {}.",
            build_info::PKG_NAME,
            build_info::PKG_VERSION,
            build_info::GIT_COMMIT_HASH.unwrap_or("unknown"),
            build_info::GIT_VERSION.unwrap_or("none"),
            dirty_flag,
        ),
        ShowableThings::Build => {
            println!(
                "This is a {} build of {}.\n\
                It was built on `{}' targetting `{}'.\n\
                The compiler used was: {}.\n\
                The CI platform used was: {}",
                build_info::PROFILE,
                build_info::PKG_NAME,
                build_info::HOST,
                build_info::TARGET,
                build_info::RUSTC_VERSION,
                build_info::CI_PLATFORM.unwrap_or("none")
            );
        }
        ShowableThings::License => println!(
            "The brot3 suite is copyright (C) 2024 {}

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>
            ",
            build_info::PKG_AUTHORS
        ),
        // git info
    };
    Ok(())
}
