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

#[derive(Serialize, Clone)]
pub struct TileError {
    serial: u64,
    error: String,
}

fn draw_tile(spec: &ViewerTileSpec, app_handle: &tauri::AppHandle) -> anyhow::Result<(), String> {
    let colourer_requested = "LogRainbow"; // TODO this will come from spec
    let colourer = colouring::decode(colourer_requested).map_err(|e| e.to_string())?;
    let engine_spec = TileSpec::try_from(spec).map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn start_tile(spec: ViewerTileSpec, app_handle: tauri::AppHandle) {
    let app_handle_copy = app_handle.clone();
    let serial = spec.serial;

    let job = tauri::async_runtime::spawn(async move {
        let serial = spec.serial;
        if let Err(error) = draw_tile(&spec, &app_handle) {
            let _ = app_handle.emit_all("tile_error", TileError { serial, error });
        }
        app_handle
            .state::<OutstandingJobs>()
            .remove_and_return(serial)
            .await;
    });
    app_handle_copy
        .state::<OutstandingJobs>()
        .add(serial, job)
        .await;
}

#[tauri::command]
pub async fn abort_tile(
    serial: u64,
    jobs: tauri::State<'_, OutstandingJobs>,
) -> Result<(), String> {
    if let Some(h) = jobs.remove_and_return(serial).await {
        h.handle.abort();
    }
    Ok(())
}
