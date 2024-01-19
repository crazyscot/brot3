// brot3 command line interface
// (c) 2024 Ross Younger
use brot3::cli::{self, list, plot};

use clap::{ArgAction, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
#[command(styles=cli::styles::get())]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output (may be repeated)
    #[arg(short, long, global(true), action = ArgAction::Count, help = "Enables debug output (may be repeated)")]
    debug: u8,

    #[arg(long, hide(true))]
    debug_cli: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Plots fractals [short form: "p"]
    #[clap(alias = "p")]
    Plot(plot::Args),
    List(list::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.debug_cli {
        println!("{:#?}", cli);
        return Ok(());
    }

    match cli.command {
        Commands::Plot(args) => plot::plot(&args, cli.debug),
        Commands::List(what) => list::list(&what),
    }
}
