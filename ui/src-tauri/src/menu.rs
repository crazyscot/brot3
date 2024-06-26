// Tauri-generated menus and handlers
// (c) 2024 Ross Younger

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu, WindowMenuEvent};

#[cfg(target_os = "macos")]
use tauri::AboutMetadata;

use crate::util::GenericError;

#[derive(Serialize, Clone)]
/// Twin of JS menu.DisplayMessageDetail
pub struct DisplayMessageDetail {
    what: String,
    detail: String,
}
impl DisplayMessageDetail {
    pub fn new(what: &str) -> DisplayMessageDetail {
        DisplayMessageDetail {
            what: what.into(),
            detail: "".into(),
        }
    }
    pub fn new_with_detail(what: &str, detail: &str) -> DisplayMessageDetail {
        DisplayMessageDetail {
            what: what.into(),
            detail: detail.into(),
        }
    }
}

pub(crate) struct ApplicationMenu {}

impl ApplicationMenu {
    pub(crate) fn new() -> ApplicationMenu {
        ApplicationMenu {}
    }

    fn cmd_or_ctrl() -> String {
        if cfg!(target_os = "macos") {
            "Cmd".to_string()
        } else {
            "Ctrl".to_string()
        }
    }

    pub(crate) fn build(&self) -> Menu {
        // work around tauri/tao accelerator misbehaviour
        #[cfg(target_os = "macos")]
        let cmd_or_ctrl = "Cmd";
        #[cfg(not(target_os = "macos"))]
        let cmd_or_ctrl = ApplicationMenu::cmd_or_ctrl();

        // Here are our custom menu items:
        let toggle_position =
            CustomMenuItem::new("toggle_position".to_string(), "Show/Hide Position")
                .accelerator(format!("{cmd_or_ctrl}+P"));
        let go_to_position = CustomMenuItem::new("go_to_position".to_string(), "Go To Position...")
            .accelerator(format!("{cmd_or_ctrl}+G"));
        let toggle_origin_centre =
            CustomMenuItem::new("toggle_origin_centre".to_string(), "Toggle Origin/Centre");
        let toggle_navigator =
            CustomMenuItem::new("toggle_navigator".to_string(), "Show/Hide Navigator");
        let save_image = CustomMenuItem::new("save_image".to_string(), "Save image...")
            .accelerator(format!("{cmd_or_ctrl}+S"));
        let save_size = CustomMenuItem::new("save_size".to_string(), "Save at size...");
        let show_max_iter = CustomMenuItem::new("show_max_iter".to_string(), "Max Iterations...")
            .accelerator(format!("{cmd_or_ctrl}+M"));

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

        let fractals = self.build_fractal_menu();

        // brot3 custom menus:
        menu.add_submenu(Submenu::new(
            "Display",
            Menu::new()
                .add_item(toggle_navigator)
                .add_native_item(MenuItem::Separator)
                .add_item(toggle_position)
                .add_item(go_to_position)
                .add_item(toggle_origin_centre),
        ))
        .add_submenu(fractals)
        .add_submenu(Submenu::new("Render", Menu::new().add_item(show_max_iter)))
        .add_submenu(Submenu::new(
            "Help",
            Menu::new().add_item(CustomMenuItem::new("show_about".to_string(), "About")),
        ))
    }

    fn build_fractal_menu(&self) -> Submenu {
        let cmd_or_ctrl = ApplicationMenu::cmd_or_ctrl();
        let standard: Vec<MenuEntry> = vec![
            MenuEntry::CustomItem(CustomMenuItem::new("select/fractal", "Select fractal...")),
            MenuEntry::NativeItem(MenuItem::Separator),
            MenuEntry::CustomItem(CustomMenuItem::new("select/colourer", "Select colourer...")),
            MenuEntry::CustomItem(
                CustomMenuItem::new("colourer/prev", "Previous colourer")
                    .accelerator(format!("{cmd_or_ctrl}+1")),
            ),
            MenuEntry::CustomItem(
                CustomMenuItem::new("colourer/next", "Next colourer")
                    .accelerator(format!("{cmd_or_ctrl}+2")),
            ),
        ];
        let menu = Menu::with_items(standard);
        Submenu::new("Fractal", menu)
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
        if id.starts_with("select/") {
            let what = id.split_at("select/".len()).1;
            self.send_message(
                event,
                "select",
                &DisplayMessageDetail::new_with_detail(what, ""),
            )
        } else {
            self.send_display_message(event, &DisplayMessageDetail::new(id))
        }
    }

    fn send_display_message(
        &self,
        event: &WindowMenuEvent,
        msg: &DisplayMessageDetail,
    ) -> anyhow::Result<()> {
        self.send_message(event, "display_message", msg)
    }
    fn send_message(
        &self,
        event: &WindowMenuEvent,
        category: &str,
        msg: &DisplayMessageDetail,
    ) -> anyhow::Result<()> {
        event.window().emit(category, msg)?;
        Ok(())
    }
}
