// List subcommand
// (c) 2024 Ross Younger
use crate::{colouring, fractal::FractalInstance, render, util::listable};

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
        ListableThings::Renderers => listable::list2::<render::Selection>(args.machine_parseable),
        ListableThings::Fractals => listable::list::<FractalInstance>(args.machine_parseable),
        ListableThings::Colourers => {
            listable::list2::<colouring::Selection>(args.machine_parseable);
        }
    };
    Ok(())
}
