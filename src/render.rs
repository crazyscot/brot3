// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod png;

use crate::fractal::Tile;
use std::error::Error;
use strum::{EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumMessage, EnumString};

/// A Renderer accepts PointData and deals with it completely.
/// This is distinct from a Palette, which accepts PointData and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render(&self, data: &Tile) -> Result<(), Box<dyn Error>>;
}

#[derive(clap::ValueEnum, Clone, Debug, Display, EnumIter, EnumString, EnumMessage)]
#[strum(serialize_all = "kebab_case")]
pub enum WhichRenderer {
    /// Comma Separated Values, one line per line of plot
    Csv,
    /// Good old ASCII art
    AsciiArt,
    /// Portable Network Graphics file
    Png,
}

pub const DEFAULT: WhichRenderer = WhichRenderer::AsciiArt;

pub fn list_vec() -> Vec<String> {
    WhichRenderer::iter().map(|a| a.to_string()).collect()
}

pub fn list(machine_parseable: bool) {
    if machine_parseable {
        println!("{:?}", list_vec());
        return;
    }

    println!("Available renderers:");
    let longest = WhichRenderer::iter()
        .map(|r| r.to_string().len())
        .max()
        .unwrap_or(1);

    let _ = WhichRenderer::iter()
        .map(|r| {
            println!(
                "  {:width$}  {}",
                r.to_string(),
                r.get_documentation().unwrap_or_default(),
                width = longest
            )
        })
        .collect::<Vec<_>>();
}

pub fn factory(selection: WhichRenderer, filename: &str) -> Box<dyn Renderer> {
    match selection {
        WhichRenderer::AsciiArt => Box::new(ascii::AsciiArt::new(filename)),
        WhichRenderer::Csv => Box::new(ascii::Csv::new(filename)),
        WhichRenderer::Png => Box::new(png::Png::new(filename)),
    }
}
