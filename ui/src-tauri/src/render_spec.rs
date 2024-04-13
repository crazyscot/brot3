// Specification of a rendering
// (c) 2024 Ross Younger

use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use brot3_engine::{
    colouring,
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
        let alg = Arc::new(fractal::factory(alg_selection));
        let col_selection =
            colouring::Selection::from_str(&input.colourer).context("colourer selection")?;
        let col = Arc::new(colouring::factory(col_selection));

        let origin = fractal::Location::Origin(input.origin.into());
        let axes = fractal::Size::AxesLength(input.axes.into());
        Ok(TileSpec::new2(
            origin,
            axes,
            Rect::new(input.width, input.height),
            &alg,
            input.maxiter,
            &col,
        ))
    }
}
