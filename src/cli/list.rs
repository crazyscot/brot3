// List subcommand
// (c) 2024 Ross Younger
use crate::render;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
enum ListableThings {
    /// Lists all available renderers
    Renderers,
    /// Lists available wombats
    Wombats,
}

/// Arguments to 'list'
#[derive(Debug, Args)]
//#[command(flatten_help = true)]
pub struct ListArgs {
    /// Machine-parseable output
    #[arg(short, long)]
    machine_parseable: bool,

    #[command(subcommand)]
    thing: ListableThings,
}

/// Implementation of 'list'
pub fn list(args: &ListArgs) -> anyhow::Result<()> {
    match args.thing {
        ListableThings::Renderers => render::list(args.machine_parseable),
        ListableThings::Wombats => {
            if args.machine_parseable {
                println!("[\"fred\",\"barney\"]");
            } else {
                println!("wombats!")
            };
        }
    }
    Ok(())
}
