// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tile_bridge;
mod viewertilespec;
use viewertilespec::ViewerTileSpec;

fn main() {
    #![allow(clippy::disallowed_types)]
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![tile])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn tile(spec: ViewerTileSpec) -> Result<tile_bridge::TileResponse, String> {
    tile_bridge::render_tile(spec).map_err(|e| e.to_string())
}
