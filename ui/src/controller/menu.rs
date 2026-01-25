//! Main menu
// (c) 2025 Ross Younger

use easy_shader_runner::egui::{self, vec2};

use crate::widgets::CheckableButton;

#[allow(unused_results)]
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
                            CheckableButton::new(&mut $var, $lbl, || false)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                                .ui(ui)
                        };
                        ($var:expr, $lbl:literal, $accel:expr) => {
                            CheckableButton::new(&mut $var, $lbl, || false)
                                .shortcut_text($accel)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                                .ui(ui)
                        };
                        ($var:expr, $lbl:literal, $accel:expr, $indet: expr) => {
                            CheckableButton::new(&mut $var, $lbl, $indet)
                                .shortcut_text($accel)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                                .ui(ui)
                        };
                    }
                    macro_rules! item {
                        ($label:expr, $accel:expr) => {
                            egui::Button::new($label)
                                .shortcut_text($accel)
                                .min_size(vec2(ITEM_WIDTH, 0.0))
                        };
                    }
                    checkbox!(self.show_controls, "Controls", "F2");
                    checkbox!(self.show_coords_window, "Data read-out", "F3");
                    checkbox!(self.show_scale_bar, "Scale bar", "F4");

                    // Fullscreen is tricky. On OSX the OS may change the state; we are not the sole
                    // arbiters.
                    if checkbox!(
                        self.fullscreen_checkbox,
                        "Fullscreen",
                        if cfg!(target_os = "macos") {
                            "^⌘F"
                        } else {
                            "F11"
                        }
                    )
                    .clicked()
                    {
                        // We need to tell easy-shader-runner explicitly about clicks
                        self.fullscreen_requested = Some(self.fullscreen_checkbox);
                    }

                    ui.separator();

                    checkbox!(self.show_fps, "Show FPS");
                    checkbox!(self.vsync, "vsync");
                    checkbox!(self.always_reiterate, "Always reiterate");
                    ui.separator();
                    if checkbox!(
                        self.perturbation_mode,
                        "Force perturbation mode",
                        "",
                        || !self.force_perturb
                    )
                    .clicked()
                    {
                        self.force_perturb = true;
                        self.reiterate = true;
                    }

                    ui.separator();
                    checkbox!(self.keyboard_help, "Show Help", "F1");

                    if ui.add(item!("About", "")).clicked() {
                        self.show_about = true;
                    }
                    ui.separator();

                    if ui
                        .add(item!(
                            "Quit",
                            if cfg!(target_os = "macos") {
                                "⌘Q"
                            } else {
                                "Ctrl+Q"
                            }
                        ))
                        .clicked()
                    {
                        // SOMEDAY: It would be tidier to call event_loop.exit().
                        std::process::exit(0);
                    }

                    // for sub menus, add a ui.menu_button(...)
                });
            });
    }
}
