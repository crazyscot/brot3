//! brot3 command line interface
// (c) 2024 Ross Younger
use clap::{ArgAction, Parser, Subcommand};

mod list;
mod plot;
mod show;
mod styles;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
#[command(styles=styles::get())]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output (may be repeated)
    #[arg(
        short,
        long,
        global(true),
        action = ArgAction::Count,
        help = "Enables debug output (may be repeated)",
        help_heading("Developer options"),
        display_order(900))
    ]
    debug: u8,

    #[arg(long, hide(true))]
    #[allow(clippy::struct_field_names)]
    debug_cli: bool,

    // include --help at the top level as an alias to the 'help' subcommand
    #[arg(
        long, hide(true), action = ArgAction::Help, required(false)
    )]
    help: Option<bool>,
}

#[derive(Debug, Subcommand)]
#[command(flatten_help = true)]
enum Commands {
    /// Plots fractals [short form: "p"]
    #[clap(alias = "p")]
    Plot(plot::Args),

    /// Lists things [short form: "l"]
    List(list::Args),

    /// Shows information about this program
    Show(show::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.debug_cli {
        println!("{cli:#?}");
        return Ok(());
    }

    match cli.command {
        Commands::Plot(args) => plot::plot(&args, cli.debug),
        Commands::List(what) => list::list(&what),
        Commands::Show(what) => show::show(&what),
    }
}
