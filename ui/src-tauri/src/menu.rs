// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
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

pub(crate) struct ApplicationMenu {
    show_zoom: AtomicBool,
    show_position: AtomicBool,
}

impl ApplicationMenu {
    pub(crate) fn new() -> ApplicationMenu {
        ApplicationMenu {
            show_zoom: true.into(),
            show_position: true.into(),
        }
    }

    pub(crate) fn build(&self) -> Menu {
        let mut toggle_zoom = CustomMenuItem::new("toggle_zoom".to_string(), "Show Zoom");
        toggle_zoom.selected = true;
        let mut toggle_position =
            CustomMenuItem::new("toggle_position".to_string(), "Show Position");
        toggle_position.selected = true;
        let go_to_position = CustomMenuItem::new("go_to_position".to_string(), "Go To Position")
            .accelerator("Ctrl+G");

        Menu::os_default("brot3")
            .add_submenu(Submenu::new(
                "Display",
                Menu::new()
                    .add_item(toggle_zoom)
                    .add_item(toggle_position)
                    .add_native_item(MenuItem::Separator)
                    .add_item(go_to_position),
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
        match id {
            "toggle_zoom" => {
                let new_state = !self.show_zoom.load(Ordering::Relaxed);
                self.show_zoom.store(new_state, Ordering::Relaxed);
                event
                    .window()
                    .menu_handle()
                    .get_item(id)
                    .set_selected(new_state)?;
            }
            "toggle_position" => {
                let new_state = !self.show_position.load(Ordering::Relaxed);
                self.show_position.store(new_state, Ordering::Relaxed);
                event
                    .window()
                    .menu_handle()
                    .get_item(id)
                    .set_selected(new_state)?;
            }
            _ => {}
        }
        self.display_message(event, id)?;
        Ok(())
    }

    fn display_message(&self, event: &WindowMenuEvent, what: &str) -> anyhow::Result<()> {
        event
            .window()
            .emit("display_message", DisplayMessageDetail::new(what))?;
        Ok(())
    }
}
