// brot3 UI bridge to engine
// (c) 2024 Ross Younger

use std::cell::RefCell;

use crate::types::{PlottedTile, TileCoordinate};

use brot3_engine::fractal::{Tile as engineTile, TileSpec as engineTileSpec};
use brot3_engine::render::Png;
use slint::{Rgba8Pixel, SharedPixelBuffer};

pub(crate) fn draw_tile(
    spec: &TileCoordinate,
    seed: Option<RefCell<engineTile>>,
) -> anyhow::Result<PlottedTile, String> {
    let engine_spec = engineTileSpec::try_from(spec).map_err(|e| e.to_string())?;
    let tile_cell = seed.unwrap_or_else(|| engineTile::new(&engine_spec, 0).into());
    // We may want a different max_iter to the seed
    let mut tile = tile_cell.borrow_mut();
    tile.spec = engine_spec;
    if tile.max_iter_plotted < tile.spec.max_iter_requested() {
        tile.plot();
    }
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(tile.spec.width(), tile.spec.height());
    let raw = buffer.make_mut_bytes();
    Png::render_rgba_into(&tile, *tile.spec.colourer(), raw);
    drop(tile);
    Ok(PlottedTile {
        tile: tile_cell,
        image: buffer,
    })
}
