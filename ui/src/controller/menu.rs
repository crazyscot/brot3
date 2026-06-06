//! Main menu
// (c) 2025 Ross Younger

use easy_shader_runner::egui::{self, vec2};

use crate::widgets::CheckableButton;

macro_rules! checkbox {
    ($ui:expr, $var:expr, $lbl:literal) => {
        CheckableButton::new(&mut $var, $lbl, || false)
            .min_size(vec2(ITEM_WIDTH, 0.0))
            .ui($ui)
    };
    ($ui:expr, $var:expr, $lbl:literal, $accel:expr) => {
        CheckableButton::new(&mut $var, $lbl, || false)
            .shortcut_text($accel)
            .min_size(vec2(ITEM_WIDTH, 0.0))
            .ui($ui)
    };
    ($ui:expr, $var:expr, $lbl:literal, $accel:expr, $indet: expr) => {
        CheckableButton::new(&mut $var, $lbl, $indet)
            .shortcut_text($accel)
            .min_size(vec2(ITEM_WIDTH, 0.0))
            .ui($ui)
    };
}

macro_rules! item {
    ($label:expr, $accel:expr) => {
        egui::Button::new($label)
            .shortcut_text($accel)
            .min_size(vec2(ITEM_WIDTH, 0.0))
    };
}

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

                    #[cfg(target_os = "macos")]
                    let ctrl_or_command = "⌘";
                    #[cfg(not(target_os = "macos"))]
                    let ctrl_or_command = "Ctrl+";
                    #[cfg(target_os = "macos")]
                    let shift_plus = "⇧";
                    #[cfg(not(target_os = "macos"))]
                    let shift_plus = "Shift+";

                    checkbox!(ui, self.show_controls, "Controls", "F2");
                    checkbox!(ui, self.show_coords_window, "Data read-out", "F3");
                    checkbox!(ui, self.show_scale_bar, "Scale bar", "F4");

                    // Fullscreen is tricky. On OSX the OS may change the state.
                    if checkbox!(
                        ui,
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

                    ui.menu_button("Debug", |ui| {
                        checkbox!(ui, self.show_fps, "Show FPS");
                        checkbox!(ui, self.show_shader, "Show selected shader");
                        checkbox!(ui, self.show_timings, "Show timings");

                        ui.separator();
                        checkbox!(ui, self.vsync, "vsync");
                        checkbox!(ui, self.always_reiterate, "Always reiterate");
                        ui.separator();
                        if checkbox!(
                            ui,
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
                    });

                    ui.separator();
                    checkbox!(ui, self.keyboard_help, "Show Help", "F1");

                    if ui.add(item!("About", "")).clicked() {
                        self.show_about = true;
                    }
                    ui.separator();

                    if ui
                        .add(item!(
                            "Open position or image...",
                            format!("{ctrl_or_command}O")
                        ))
                        .clicked()
                    {
                        self.show_open = true;
                    }
                    ui.separator();
                    if ui
                        .add(item!("Save image...", format!("{ctrl_or_command}S")))
                        .clicked()
                    {
                        self.show_save = true;
                    }
                    if ui
                        .add(item!(
                            "Save position...",
                            format!("{ctrl_or_command}{shift_plus}S")
                        ))
                        .clicked()
                    {
                        self.show_save_position = true;
                    }
                    ui.separator();

                    if ui
                        .add(item!("Quit", format!("{ctrl_or_command}Q")))
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
