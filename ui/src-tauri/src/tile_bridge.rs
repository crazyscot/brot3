// Viewer tile to PNG mapping
// (c) 2024 Ross Younger

use super::ViewerTileSpec;
use brot3_engine::{
    colouring,
    fractal::{Tile, TileSpec},
    render,
};

use serde::Serialize;
use tauri::Manager;

#[derive(Serialize, Clone)]
pub struct TileResponse {
    serial: u64,
    rgba_blob: bytes::Bytes,
}

pub fn start_tile_render(spec: ViewerTileSpec, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let colourer_requested = "LogRainbow"; // TODO this will come from spec
    let colourer = colouring::decode(colourer_requested)?;

    let engine_spec = TileSpec::try_from(&spec)?;
    ::tauri::async_runtime::spawn(async move {
        let mut tile = Tile::new(&engine_spec, 0);
        tile.plot(512); // TODO specify max_iter, or even go dynamic
        app_handle
            .emit_all(
                "tile_complete",
                TileResponse {
                    serial: spec.serial,
                    rgba_blob: render::as_rgba(&tile, colourer).into(),
                },
            )
            .unwrap();
    });
    Ok(())
}
