// Viewer tile to PNG mapping
// (c) 2024 Ross Younger

use super::ViewerTileSpec;
use crate::OutstandingJobs;
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

#[tauri::command]
pub fn start_tile(spec: ViewerTileSpec, app_handle: tauri::AppHandle) -> Result<(), String> {
    let serial = spec.serial;
    let colourer_requested = "LogRainbow"; // TODO this will come from spec
    let colourer = colouring::decode(colourer_requested).map_err(|e| e.to_string())?;
    let engine_spec = TileSpec::try_from(&spec).map_err(|e| e.to_string())?;

    let app_handle_for_cb = app_handle.clone();

    let job = ::tauri::async_runtime::spawn_blocking(move || {
        let mut tile = Tile::new(&engine_spec, 0);
        tile.plot(512); // TODO specify max_iter, or even go dynamic
        let jobs = app_handle_for_cb.state::<OutstandingJobs>();
        app_handle_for_cb
            .emit_all(
                "tile_complete",
                TileResponse {
                    serial: spec.serial,
                    rgba_blob: render::as_rgba(&tile, colourer).into(),
                },
            )
            .unwrap_or_else(|e| println!("Error notifying: {e}"));
        jobs.remove_and_return(serial);
    });

    app_handle.state::<OutstandingJobs>().add(serial, job);
    Ok(())
}

#[tauri::command]
pub fn abort_tile(serial: u64, jobs: tauri::State<OutstandingJobs>) -> Result<(), String> {
    if let Some(h) = jobs.remove_and_return(serial) {
        h.abort();
    }
    Ok(())
}
