// Tauri-facing side of brot3
// (c) 2024 Ross Younger

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod jobs;
mod menu;
mod tile_bridge;
mod viewertilespec;

use jobs::OutstandingJobs;
use viewertilespec::ViewerTileSpec;

fn main() {
    #![allow(clippy::disallowed_types)]
    tauri::Builder::default()
        .manage(OutstandingJobs::default())
        .invoke_handler(tauri::generate_handler![
            tile_bridge::start_tile,
            tile_bridge::abort_tile,
            tile_bridge::get_metadata,
        ])
        .menu(menu::make_menu())
        .on_menu_event(menu::on_menu)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
