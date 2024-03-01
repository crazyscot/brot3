// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowMenuEvent};

pub(crate) fn make_menu() -> Menu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let submenu = Submenu::new("File", Menu::new().add_item(quit));
    Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_submenu(submenu)
}

pub(crate) fn on_menu(event: WindowMenuEvent) {
    #[allow(clippy::single_match)]
    match event.menu_item_id() {
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}
