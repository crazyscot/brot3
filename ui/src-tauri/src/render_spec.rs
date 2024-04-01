// Specification of a rendering
// (c) 2024 Ross Younger

use std::str::FromStr;

use anyhow::Context;
use brot3_engine::{
    fractal::{self, TileSpec},
    util::Rect,
};
use serde::{Deserialize, Serialize};

use crate::tile_bridge::SerializablePoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSpec {
    /// Origin of this tile (bottom-left corner, smallest real/imaginary coefficients)
    pub origin: SerializablePoint,
    /// Axes length for this tile
    pub axes: SerializablePoint,
    /// Iteration limit
    pub maxiter: u32,
    /// The name of the algorithm to use
    pub algorithm: String,
    /// The name of the colourer to use
    pub colourer: String,
    /// Desired render width in pixels
    pub width: u32,
    /// Desired render height in pixels
    pub height: u32,
}

impl TryFrom<RenderSpec> for TileSpec {
    type Error = anyhow::Error;

    fn try_from(input: RenderSpec) -> Result<Self, Self::Error> {
        let alg_selection =
            fractal::Selection::from_str(&input.algorithm).context("fractal selection")?;
        let alg = fractal::factory(alg_selection);

        Ok(TileSpec::new(
            input.origin.into(),
            input.axes.into(),
            Rect::new(input.width, input.height),
            alg,
        ))
    }
}
