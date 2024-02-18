// Viewer tile to PNG mapping
// (c) 2024 Ross Younger

use super::ViewerTileSpec;
use brot3_engine::{
    colouring,
    fractal::{Tile, TileSpec},
    render,
};

use serde::Serialize;

#[derive(Serialize)]
pub struct RGBABlob {
    serial: u64,
    blob: bytes::Bytes,
}

pub fn render_tile(spec: ViewerTileSpec) -> anyhow::Result<RGBABlob> {
    let colourer_requested = "LogRainbow"; // TODO this will come from spec
    let colourer = colouring::decode(colourer_requested)?;

    let engine_spec = TileSpec::try_from(&spec)?;
    let mut tile = Tile::new(&engine_spec, 0);
    tile.plot(512); // TODO specify max_iter, or even go dynamic

    Ok(RGBABlob {
        serial: spec.serial,
        blob: render::as_rgba(&tile, colourer).into(),
    })
}
