// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tile_bridge;
mod viewertilespec;
use viewertilespec::ViewerTileSpec;

fn main() {
    #![allow(clippy::disallowed_types)]
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_tile])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_tile(spec: ViewerTileSpec, app_handle: tauri::AppHandle) -> Result<(), String> {
    tile_bridge::start_tile_render(spec, app_handle).map_err(|e| e.to_string())
}
