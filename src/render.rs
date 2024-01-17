/// Conversion of fractal PointData into its output format
/// (c) 2024 Ross Younger
pub mod ascii;

use crate::fractal::Tile;
use std::error::Error;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

/// A Renderer accepts PointData and deals with it completely.
/// This is distinct from a Palette, which accepts PointData and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render(&self, data: &Tile) -> Result<(), Box<dyn Error>>;
}

#[derive(clap::ValueEnum, Clone, Debug, Display, EnumIter, EnumString)]
pub enum WhichRenderer {
    Csv,
    AsciiArt,
}

pub const DEFAULT: WhichRenderer = WhichRenderer::AsciiArt;

pub fn list() -> Vec<String> {
    WhichRenderer::iter().map(|a| a.to_string()).collect()
}

pub fn factory(selection: WhichRenderer, filename: &str) -> Box<dyn Renderer> {
    match selection {
        WhichRenderer::AsciiArt => Box::new(ascii::AsciiArt::new(filename)),
        WhichRenderer::Csv => Box::new(ascii::Csv::new(filename)),
    }
}
