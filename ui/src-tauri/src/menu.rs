// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowMenuEvent};

#[derive(Serialize, Clone)]
pub struct GenericError {
    error: String,
}

pub(crate) struct ApplicationMenu {}

impl ApplicationMenu {
    pub(crate) fn new() -> ApplicationMenu {
        ApplicationMenu {}
    }

    pub(crate) fn build(&self) -> Menu {
        Menu::os_default("brot3")
            .add_submenu(Submenu::new(
                "Help",
                Menu::new().add_item(CustomMenuItem::new("about".to_string(), "About")),
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
        #[allow(clippy::single_match)]
        match event.menu_item_id() {
            "about" => {
                event.window().emit("showAbout", ())?;
            }
            _ => {}
        }
        Ok(())
    }
}
