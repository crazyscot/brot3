//! Keyboard input handling
//!
//! Portions of this file are based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#![allow(unused_results)]

use easy_shader_runner::{
    egui,
    winit::{
        event::KeyEvent,
        keyboard::{Key, NamedKey},
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
                } else {
                    minimax(&mut self.movement.$id, 0., 0., increase);
                }
            }
        )*
    };
}

impl super::Controller {
    field_fn!(gradient, offset, gamma, saturation, lightness);

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
                        row!("E R", "Exponent Re");
                        row!("D F", "Exponent Im");
                        row!("Y U", "Gradient");
                        row!("H J", "Offset");
                        row!("N M", "Gamma");
                        row!("I O", "Saturation");
                        row!("K L", "Lightness");

                        ui.separator();
                        ui.separator();
                        ui.end_row(); // blank line
                        row!("Shift", "Speed up");
                        row!("Alt", "Slow down");
                        row!("Ctrl", "... more");
                    });
            });
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn keyboard_input_impl(&mut self, key: &KeyEvent) {
        use easy_shader_runner::winit::platform::modifier_supplement::KeyEventExtModifierSupplement as _;

        let pressed = key.state.is_pressed();
        match key.logical_key {
            Key::Named(NamedKey::Control) => {
                self.ctrl_pressed = pressed;
            }
            Key::Named(NamedKey::Shift) => {
                self.shift_pressed = pressed;
            }
            Key::Named(NamedKey::Alt) => {
                self.alt_pressed = pressed;
            }
            Key::Named(NamedKey::Super) => {
                self.super_pressed = pressed;
            }
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
                if self.ctrl_pressed {
                    // Perf test mode (undocumented) is Ctrl+F11 on all platforms.
                    self.fullscreen_requested = Some(true);
                    self.vsync = false;
                    self.show_fps = true;
                    self.show_controls = false;
                    self.show_coords_window = false;
                    self.show_scale_bar = false;
                    self.always_reiterate = true;
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
                'f' if self.ctrl_pressed && self.super_pressed => {}
                'd' | 'f' => self.expo_im(c == 'f', pressed),

                // Quit
                #[cfg(target_os = "macos")]
                'q' if pressed && self.super_pressed => std::process::exit(0),
                // SOMEDAY: It would be tidier to call event_loop.exit().
                // Expose this in easy-shader-runner, or add a new CustomEvent
                // and expose an EventLoopProxy.
                #[cfg(not(target_os = "macos"))]
                'q' if pressed && self.ctrl_pressed => std::process::exit(0),

                'y' | 'u' => self.gradient(c == 'u', pressed),
                'h' | 'j' => self.offset(c == 'j', pressed),
                'n' | 'm' => self.gamma(c == 'm', pressed),
                'i' | 'o' => self.saturation(c == 'o', pressed),
                'k' | 'l' => self.lightness(c == 'l', pressed),
                'a' => self.show_about = true,
                's' if pressed && self.ctrl_pressed => {
                    if self.shift_pressed {
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
        } else {
            minimax(&mut self.movement.exponent, 0., 0., increase);
        }
    }

    fn expo_im(&mut self, increase: bool, active: bool) {
        if active {
            let magnitude = self.state.exponent.ui_step();
            let sign = if increase { 1. } else { -1. };
            self.movement.exponent_im = sign * magnitude;
        } else {
            minimax(&mut self.movement.exponent_im, 0., 0., increase);
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

/// Clamps a value with either a minimum or a maximum.
fn minimax<T>(value: &mut T, min: T, max: T, clamp_max: bool)
where
    T: Copy + core::cmp::PartialOrd,
{
    if clamp_max {
        *value = num_traits::clamp_max(*value, max);
    } else {
        *value = num_traits::clamp_min(*value, min);
    }
}
