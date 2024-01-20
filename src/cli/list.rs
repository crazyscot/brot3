// List subcommand
// (c) 2024 Ross Younger
use crate::{fractal::SelectionF, render::SelectionR, util::listable};

#[derive(Debug, clap::Subcommand)]
enum ListableThings {
    /// Lists all available renderers
    #[clap(alias = "r")]
    Renderers,
    /// Lists available fractal algorithms
    #[clap(alias = "f")]
    Fractals,
}

/// Arguments to 'list'
#[derive(Debug, clap::Args)]
//#[command(flatten_help = true)]
pub struct Args {
    /// Machine-parseable output
    #[arg(short, long)]
    machine_parseable: bool,

    #[command(subcommand)]
    thing: ListableThings,
}

/// Implementation of 'list'
pub fn list(args: &Args) -> anyhow::Result<()> {
    match args.thing {
        ListableThings::Renderers => listable::list::<SelectionR>(args.machine_parseable),
        ListableThings::Fractals => listable::list::<SelectionF>(args.machine_parseable),
    }
    Ok(())
}
