//! Main menu
// (c) 2025 Ross Younger

use easy_shader_runner::egui::{self, vec2};

use crate::widgets::CheckableButton;

impl super::Controller {
    pub(crate) fn main_menu(&mut self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new("mainmenu"))
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10., 10.))
            .show(ctx, |ui| {
                let img = egui::Image::from(egui::include_image!("../../misc/hamburger_icon.png"))
                    .fit_to_exact_size(vec2(20.0, 20.0));
                ui.menu_image_button(img, |ui| {
                    const ITEM_WIDTH: f32 = 120.0;
                    use egui::Widget as _;

                    macro_rules! checkbox {
                        ($var:expr, $lbl:literal) => {
                            CheckableButton::new(&mut $var, $lbl)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                                .ui(ui);
                        };
                        ($var:expr, $lbl:literal, $accel:literal) => {
                            CheckableButton::new(&mut $var, $lbl)
                                .shortcut_text($accel)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                                .ui(ui);
                        };
                    }
                    macro_rules! item {
                        ($label:expr, $accel:literal) => {
                            egui::Button::new($label)
                                .shortcut_text($accel)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                        };
                    }
                    checkbox!(self.show_controls, "Controls", "F2");
                    checkbox!(self.show_coords_window, "Data read-out", "F3");
                    checkbox!(self.show_scale_bar, "Scale bar", "F4");
                    checkbox!(self.fullscreen_requested, "Fullscreen", "F11");

                    ui.separator();

                    checkbox!(self.show_fps, "Show FPS");
                    checkbox!(self.vsync, "vsync");

                    ui.separator();
                    checkbox!(self.keyboard_help, "Show Help", "F1");

                    if ui.add(item!("About", "")).clicked() {
                        self.show_about = true;
                    }
                    ui.separator();

                    if ui.add(item!("Quit", "Ctrl+Q")).clicked() {
                        // SOMEDAY: It would be tidier to call event_loop.exit().
                        std::process::exit(0);
                    }

                    // for sub menus, add a ui.menu_button(...)
                });
            });
    }
}
