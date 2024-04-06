// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowMenuEvent};

#[cfg(target_os = "macos")]
use tauri::AboutMetadata;

use crate::util::GenericError;

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
        // work around tauri/tao accelerator misbehaviour
        #[cfg(target_os = "macos")]
        let cmd_or_ctrl = "Cmd";
        #[cfg(not(target_os = "macos"))]
        let cmd_or_ctrl = "Ctrl";

        // Here are our custom menu items:
        let toggle_position =
            CustomMenuItem::new("toggle_position".to_string(), "Show/Hide Position")
                .accelerator(format!("{cmd_or_ctrl}+P"));
        let go_to_position = CustomMenuItem::new("go_to_position".to_string(), "Go To Position...")
            .accelerator(format!("{cmd_or_ctrl}+G"));
        let toggle_origin_centre =
            CustomMenuItem::new("toggle_origin_centre".to_string(), "Toggle Origin/Centre");
        let save_image = CustomMenuItem::new("save_image".to_string(), "Save image...")
            .accelerator(format!("{cmd_or_ctrl}+S"));
        let save_size = CustomMenuItem::new("save_size".to_string(), "Save at size...");

        // menu::os_default is lame in tauri1, doesn't support modifying the default menus.
        // For now we will clone and hack. TODO(tauri2) - overhaul this.
        #[allow(unused)]
        let app_name = "brot3";
        let mut menu = Menu::new();
        #[cfg(target_os = "macos")]
        {
            menu = menu.add_submenu(Submenu::new(
                app_name,
                Menu::new()
                    .add_native_item(MenuItem::About(
                        app_name.to_string(),
                        AboutMetadata::default(),
                    ))
                    .add_native_item(MenuItem::Separator)
                    .add_native_item(MenuItem::Services)
                    .add_native_item(MenuItem::Separator)
                    .add_native_item(MenuItem::Hide)
                    .add_native_item(MenuItem::HideOthers)
                    .add_native_item(MenuItem::ShowAll)
                    .add_native_item(MenuItem::Separator)
                    .add_native_item(MenuItem::Quit),
            ));
        }

        let mut file_menu = Menu::new();
        // brot3 custom items:
        file_menu = file_menu
            .add_item(save_image)
            .add_item(save_size)
            .add_native_item(MenuItem::Separator);

        file_menu = file_menu.add_native_item(MenuItem::CloseWindow);
        #[cfg(not(target_os = "macos"))]
        {
            file_menu = file_menu.add_native_item(MenuItem::Quit);
        }
        menu = menu.add_submenu(Submenu::new("File", file_menu));

        #[cfg(not(target_os = "linux"))]
        let mut edit_menu = Menu::new();
        #[cfg(target_os = "macos")]
        {
            edit_menu = edit_menu.add_native_item(MenuItem::Undo);
            edit_menu = edit_menu.add_native_item(MenuItem::Redo);
            edit_menu = edit_menu.add_native_item(MenuItem::Separator);
        }
        #[cfg(not(target_os = "linux"))]
        {
            edit_menu = edit_menu.add_native_item(MenuItem::Cut);
            edit_menu = edit_menu.add_native_item(MenuItem::Copy);
            edit_menu = edit_menu.add_native_item(MenuItem::Paste);
        }
        #[cfg(target_os = "macos")]
        {
            edit_menu = edit_menu.add_native_item(MenuItem::SelectAll);
        }
        #[cfg(not(target_os = "linux"))]
        {
            menu = menu.add_submenu(Submenu::new("Edit", edit_menu));
        }
        #[cfg(target_os = "macos")]
        {
            menu = menu.add_submenu(Submenu::new(
                "View",
                Menu::new().add_native_item(MenuItem::EnterFullScreen),
            ));
        }
        let mut window_menu = Menu::new();
        window_menu = window_menu.add_native_item(MenuItem::Minimize);
        #[cfg(target_os = "macos")]
        {
            window_menu = window_menu.add_native_item(MenuItem::Zoom);
            window_menu = window_menu.add_native_item(MenuItem::Separator);
        }
        window_menu = window_menu.add_native_item(MenuItem::CloseWindow);
        menu = menu.add_submenu(Submenu::new("Window", window_menu));

        // brot3 custom menus:
        menu.add_submenu(Submenu::new(
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
