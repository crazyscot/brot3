// brot3 UI bridge to engine
// (c) 2024 Ross Younger

use crate::types::TileCoordinate;

use brot3_engine::{
    fractal::{Tile, TileSpec},
    render::Png,
};
use slint::Rgba8Pixel;

pub(crate) fn draw_tile(
    spec: &TileCoordinate,
) -> anyhow::Result<slint::SharedPixelBuffer<Rgba8Pixel>, String> {
    let engine_spec = TileSpec::try_from(spec).map_err(|e| e.to_string())?;
    let mut new_tile = Tile::new(&engine_spec, 0);
    new_tile.plot();
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(
        engine_spec.width(),
        engine_spec.height(),
    );
    let raw = buffer.make_mut_bytes();
    Png::render_rgba_into(&new_tile, *engine_spec.colourer(), raw);
    Ok(buffer)
}
