// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod framework;
mod png;

#[allow(clippy::module_name_repetitions)]
pub use framework::{factory, RenderInstance, Renderer, Selection};
pub use png::Png; // to allow direct benchmarking

use crate::colouring::Instance;
use crate::fractal::Tile;

/// Renders a tile as a low-level array of RGBA values.
/// These are returned in the obvious left to right, top to bottom order.
#[must_use]
pub fn as_rgba(tile: &Tile, colourer: Instance) -> Vec<u8> {
    png::Png::render_rgba(tile, colourer)
}
