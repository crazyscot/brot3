// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowMenuEvent};

#[derive(Serialize, Clone)]
pub struct GenericError {
    error: String,
}

#[derive(Serialize, Clone)]
/// Twin of JS menu.DisplayMessageDetail
pub struct DisplayMessageDetail {
    what: String,
}
impl DisplayMessageDetail {
    pub fn new(what: &str) -> DisplayMessageDetail {
        DisplayMessageDetail {
            what: what.to_string(),
        }
    }
}

pub(crate) struct ApplicationMenu {}

impl ApplicationMenu {
    pub(crate) fn new() -> ApplicationMenu {
        ApplicationMenu {}
    }

    pub(crate) fn build(&self) -> Menu {
        let toggle_position =
            CustomMenuItem::new("toggle_position".to_string(), "Show/Hide Position")
                .accelerator("Ctrl+P");
        let go_to_position = CustomMenuItem::new("go_to_position".to_string(), "Go To Position...")
            .accelerator("Ctrl+G");
        let toggle_origin_centre =
            CustomMenuItem::new("toggle_origin_centre".to_string(), "Toggle Origin/Centre");

        Menu::os_default("brot3")
            .add_submenu(Submenu::new(
                "Display",
                Menu::new()
                    .add_item(toggle_position)
                    .add_item(go_to_position)
                    .add_native_item(MenuItem::Separator)
                    .add_item(toggle_origin_centre),
            ))
            .add_submenu(Submenu::new(
                "Help",
                Menu::new().add_item(CustomMenuItem::new("show_about".to_string(), "About")),
            ))
    }

    pub(crate) fn on_menu(&self, event: WindowMenuEvent) {
        if let Err(error) = self.on_menu_guts(&event) {
            println!("Error: {error}");
            let _ = event.window().emit_all(
                "genericError",
                GenericError {
                    error: error.to_string(),
                },
            ); // if this goes wrong, ¯\_(ツ)_/¯
        }
    }

    fn on_menu_guts(&self, event: &WindowMenuEvent) -> anyhow::Result<()> {
        let id = event.menu_item_id();
        self.display_message(event, id)
    }

    fn display_message(&self, event: &WindowMenuEvent, what: &str) -> anyhow::Result<()> {
        event
            .window()
            .emit("display_message", DisplayMessageDetail::new(what))?;
        Ok(())
    }
}
