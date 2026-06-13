//! Keyboard input handling
//!
//! Portions of this file are based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#![allow(unused_results)]

use easy_shader_runner::{
    egui,
    winit::{
        event::{KeyEvent, Modifiers},
        keyboard::{Key, ModifiersState, NamedKey},
    },
};

const MOVE_SPEED: f64 = 0.4;
const ZOOM_SPEED: f64 = 3.0;

/// DRY... Define a standard field function
macro_rules! field_fn {
    ($($id:ident), *) => {
        $(
            fn $id(&mut self, increase: bool, active: bool) {
                if active {
                    let sign = if increase { 1. } else { -1. };
                    self.movement.$id = sign * 0.1;
                    // The slider clamps the value, so we don't need to worry about it here.
                }
            }
        )*
    };
}

pub(crate) trait ModifiersExt {
    // Check control, or command, as appropriate to the OS
    fn ctrl_or_cmd(&self) -> bool;
    fn shift(&self) -> bool;
    fn ctrl(&self) -> bool;
    fn alt(&self) -> bool;
    #[cfg(target_os = "macos")]
    fn super_(&self) -> bool;
}

impl ModifiersExt for ModifiersState {
    // Check control, or command, as appropriate to the OS
    fn ctrl_or_cmd(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.contains(ModifiersState::SUPER)
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.contains(ModifiersState::CONTROL)
        }
    }

    fn shift(&self) -> bool {
        self.contains(ModifiersState::SHIFT)
    }

    fn ctrl(&self) -> bool {
        self.contains(ModifiersState::CONTROL)
    }

    fn alt(&self) -> bool {
        self.contains(ModifiersState::ALT)
    }

    #[cfg(target_os = "macos")]
    fn super_(&self) -> bool {
        self.contains(ModifiersState::SUPER)
    }
}

impl super::Controller {
    field_fn!(gradient, offset);

    pub(super) fn keyboard_help_window(ctx: &egui::Context) {
        egui::Window::new("keyboard")
            //.default_width(crate::controller::ui::DEFAULT_WIDTH)
            //.auto_sized()
            .default_width(100.)
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::splat(-10.))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Keyboard").size(15.0));
                ui.separator();
                egui::Grid::new("keyboard grid")
                    .striped(true)
                    .show(ui, |ui| {
                        macro_rules! row {
                            ($key:literal, $lbl:literal) => {
                                ui.label($key);
                                ui.label($lbl);
                                ui.end_row();
                            };
                        }
                        row!("F7 F8", "Fractal");
                        row!("F9 F10", "Palette");

                        ui.separator();
                        ui.separator();
                        ui.end_row(); // blank line

                        row!("⬅➡", "Real");
                        row!("⬆⬇", "Complex");
                        row!("Z X", "Zoom");
                        row!("E R", "Exponent");
                        row!("Y U", "Gradient");
                        row!("H J", "Offset");

                        ui.separator();
                        ui.separator();
                        ui.end_row(); // blank line
                        row!("Shift", "Speed up");
                        row!("Alt", "Slow down");
                        row!("Ctrl", "... more");
                    });
            });
    }

    pub(super) fn modifiers_changed_impl(&mut self, mods: Modifiers) {
        self.keyboard_modifiers = mods.state();
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn keyboard_input_impl(&mut self, key: &KeyEvent) {
        use easy_shader_runner::winit::platform::modifier_supplement::KeyEventExtModifierSupplement as _;

        let pressed = key.state.is_pressed();
        match key.logical_key {
            Key::Named(NamedKey::ArrowLeft) => {
                if pressed {
                    self.movement.translate.x = -MOVE_SPEED;
                } else {
                    self.movement.translate.x = self.movement.translate.x.max(0.0);
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                if pressed {
                    self.movement.translate.x = MOVE_SPEED;
                } else {
                    self.movement.translate.x = self.movement.translate.x.min(0.0);
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                if pressed {
                    self.movement.translate.y = -MOVE_SPEED;
                } else {
                    self.movement.translate.y = self.movement.translate.y.max(0.0);
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                if pressed {
                    self.movement.translate.y = MOVE_SPEED;
                } else {
                    self.movement.translate.y = self.movement.translate.y.min(0.0);
                }
            }
            Key::Named(NamedKey::F1) if pressed => {
                self.keyboard_help = !self.keyboard_help;
            }
            Key::Named(NamedKey::F2) if pressed => {
                self.show_controls = !self.show_controls;
            }
            Key::Named(NamedKey::F3) if pressed => {
                self.show_coords_window = !self.show_coords_window;
            }
            Key::Named(NamedKey::F4) if pressed => {
                self.show_scale_bar = !self.show_scale_bar;
            }
            Key::Named(NamedKey::F5) if pressed => {
                self.show_fps = !self.show_fps;
            }
            Key::Named(NamedKey::F7) if pressed => {
                self.fractal(false);
            }
            Key::Named(NamedKey::F8) if pressed => {
                self.fractal(true);
            }
            Key::Named(NamedKey::F9) if pressed => {
                self.palette(false);
            }
            Key::Named(NamedKey::F10) if pressed => {
                self.palette(true);
            }

            Key::Named(NamedKey::F11) if pressed => {
                if self.keyboard_modifiers.ctrl() {
                    // Perf test mode (undocumented) is Ctrl+F11 on all platforms.
                    self.fullscreen_requested = Some(true);
                    self.vsync = false;
                    self.show_fps = true;
                    self.show_controls = false;
                    self.show_coords_window = false;
                    self.show_scale_bar = false;
                    self.always_reiterate = true;
                    self.show_timings = true;
                } else if cfg!(not(target_os = "macos")) {
                    // F11 only operates fullscreen on Windows and Linux; Apple uses Ctrl+Cmd+F, and
                    // that's implemented by the OS.
                    self.fullscreen_requested = Some(!self.fullscreen_checkbox);
                }
            }
            _ => (),
        }
        if let Key::Character(c) = key.key_without_modifiers() {
            let Some(c) = c.chars().next() else {
                return; /* should never happen */
            };
            match c {
                'z' | 'x' => self.kbd_zoom(c == 'z', pressed),
                'e' | 'r' => self.expo_re(c == 'r', pressed),
                #[cfg(target_os = "macos")]
                // Fullscreen on Apple is implemented by the OS
                'f' if ctrl && self.keyboard_modifiers.super_() => {}

                // Quit
                'q' if pressed && self.keyboard_modifiers.ctrl_or_cmd() => std::process::exit(0),

                'y' | 'u' => self.gradient(c == 'u', pressed),
                'h' | 'j' => self.offset(c == 'j', pressed),
                'o' if pressed && self.keyboard_modifiers.ctrl_or_cmd() => self.show_open = true,
                'a' => self.show_about = true,
                's' if pressed && self.keyboard_modifiers.ctrl_or_cmd() => {
                    if self.keyboard_modifiers.shift() {
                        self.show_save_position = true;
                    } else {
                        self.show_save = true;
                    }
                }
                _ => {}
            }
            // Remember to add new keys to keyboard_help_window !
        }
    }

    fn kbd_zoom(&mut self, inwards: bool, active: bool) {
        if active {
            self.movement.zoom2 = if inwards { ZOOM_SPEED } else { -ZOOM_SPEED };
        } else if inwards {
            /* !active inwards */
            if self.movement.zoom2 > 1. {
                self.movement.zoom2 = 1.;
            }
        } else if self.movement.zoom2 < 1. {
            /* !active !inwards */
            self.movement.zoom2 = 1.;
        }
    }

    fn expo_re(&mut self, increase: bool, active: bool) {
        if active {
            let magnitude = self.state.exponent.ui_step();
            let sign = if increase { 1. } else { -1. };
            self.movement.exponent = sign * magnitude;
            // no need to clamp the value; the slider does it for us
        }
    }

    fn fractal(&mut self, increment: bool) {
        let delta = if increment { 1 } else { -1 };
        self.state.algorithm += delta;
        self.reiterate = true;
    }

    fn palette(&mut self, increment: bool) {
        let delta = if increment { 1 } else { -1 };
        self.state.palette.colourer += delta;
    }
}
