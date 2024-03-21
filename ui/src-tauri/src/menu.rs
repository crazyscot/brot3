// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowMenuEvent};

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
        let mut show_zoom = CustomMenuItem::new("show_zoom".to_string(), "Show Zoom");
        show_zoom.selected = true;

        Menu::os_default("brot3")
            .add_submenu(Submenu::new("View", Menu::new().add_item(toggle_zoom)))
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
        match event.menu_item_id() {
            "about" => {
                event.window().emit("showAbout", ())?;
            }
            "show_zoom" => {
                self.display_message(event, "toggle_zoom")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn display_message(&self, event: &WindowMenuEvent, what: &str) -> anyhow::Result<()> {
        event
            .window()
            .emit("display_message", DisplayMessageDetail::new(what))?;
        Ok(())
    }
}
