/// Conversion of fractal PointData into its output format
/// (c) 2024 Ross Younger
pub mod ascii;

use std::error::Error;

use crate::fractal::Tile;

/// A Renderer accepts PointData and deals with it completely.
/// This is distinct from a Palette, which accepts PointData and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render(&self, data: &Tile) -> Result<(), Box<dyn Error>>;
}
