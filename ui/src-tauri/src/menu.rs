// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowMenuEvent};

#[derive(Serialize, Clone)]
pub struct GenericError {
    error: String,
}

pub(crate) fn make_menu() -> Menu {
    let about = CustomMenuItem::new("about".to_string(), "About");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_submenu(Submenu::new("File", Menu::new().add_item(quit)))
        .add_submenu(Submenu::new("Help", Menu::new().add_item(about)))
}

pub(crate) fn on_menu(event: WindowMenuEvent) {
    if let Err(error) = on_menu_guts(&event) {
        println!("Error: {error}");
        let _ = event.window().emit_all(
            "genericError",
            GenericError {
                error: error.to_string(),
            },
        ); // if this goes wrong, ¯\_(ツ)_/¯
    }
}

fn on_menu_guts(event: &WindowMenuEvent) -> anyhow::Result<()> {
    #[allow(clippy::single_match)]
    match event.menu_item_id() {
        "about" => {
            event.window().emit("showAbout", ())?;
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
    Ok(())
}
