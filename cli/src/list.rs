// List subcommand
// (c) 2024 Ross Younger
use brot3_engine::{
    colouring::Colourer,
    fractal::{self},
    render::Renderer,
    util::Listable as _,
};

use clap::ArgAction;

#[derive(Debug, clap::Subcommand)]
enum ListableThings {
    /// Lists available fractal algorithms
    #[clap(alias = "f")]
    Fractals,
    /// Lists available colouring algorithms
    #[clap(alias = "c")]
    Colourers,
    /// Lists all available output file types
    #[clap(alias = "t", name = "output-types")]
    Renderers,
}

/// Arguments to 'list'
#[derive(Debug, clap::Args)]
#[command(flatten_help = true)]
pub struct Args {
    #[arg(
        long, hide(true), action = ArgAction::Help, required(false)
    )]
    help: Option<bool>,

    #[command(subcommand)]
    thing: ListableThings,
}

/// Implementation of 'list'
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn list(args: &Args) -> anyhow::Result<()> {
    match args.thing {
        ListableThings::Renderers => Renderer::print_list(),
        ListableThings::Fractals => fractal::Selection::print_list(),
        ListableThings::Colourers => {
            Colourer::print_list();
        }
    }
    Ok(())
}
