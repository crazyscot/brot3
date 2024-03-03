// List subcommand
// (c) 2024 Ross Younger
use brot3_engine::{
    colouring,
    fractal::{self},
    render,
    util::listable,
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
//#[command(flatten_help = true)]
pub(crate) struct Args {
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
        ListableThings::Renderers => listable::print_list::<render::Selection>(),
        ListableThings::Fractals => listable::print_list::<fractal::Selection>(),
        ListableThings::Colourers => {
            listable::print_list::<colouring::Selection>();
        }
    };
    Ok(())
}
