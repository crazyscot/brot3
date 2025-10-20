use easy_shader_runner::egui;
use easy_shader_runner::winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

const MOVE_SPEED: f64 = 0.2;
const ZOOM_SPEED: f64 = 1.4;

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
    pub(super) fn keyboard_help_window(&mut self, ctx: &egui::Context) {
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
                        row!("⬆⬇⬅➡", "Move");
                        row!("Z X", "Zoom");
                        row!("E R", "Exponent");
                        row!("[ ]", "Palette");
                        row!("Y U", "Gradient");
                        row!("H J", "Offset");
                        row!("N M", "Gamma");
                        row!("I O", "Saturation");
                        row!("K L", "Lightness");
                        row!("T", "Toggle UI");
                        row!("F11", "Fullscreen");
                        row!("^Q | Esc", "Quit");
                    });
            });
    }

    pub(super) fn keyboard_input_impl(&mut self, key: KeyEvent) {
        let pressed = key.state.is_pressed();
        match key.logical_key {
            Key::Named(NamedKey::Control) => {
                self.ctrl_pressed = pressed;
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
            Key::Named(NamedKey::F11) => {
                if pressed {
                    self.fullscreen_requested = !self.fullscreen_requested;
                }
            }
            Key::Character(c) => {
                let c = match c.chars().next() {
                    Some(ch) => ch,
                    None => return, // should never happen
                };
                match c {
                    'z' | 'x' => self.kbd_zoom(c == 'z', pressed),
                    'e' | 'r' => self.expo(c == 'r', pressed),
                    'q' if pressed && self.ctrl_pressed => std::process::exit(0),
                    // N.B. Escape also instructs easy-shader-runner to quit.
                    // SOMEDAY: It would be tidier to call event_loop.exit().
                    // Expose this in easy-shader-runner, or add a new CustomEvent
                    // and expose an EventLoopProxy.
                    't' if pressed => self.show_ui = !self.show_ui,
                    '[' | ']' if pressed => self.palette(c == ']'),
                    'y' | 'u' => self.gradient(c == 'u', pressed),
                    'h' | 'j' => self.offset(c == 'j', pressed),
                    'n' | 'm' => self.gamma(c == 'm', pressed),
                    'i' | 'o' => self.saturation(c == 'o', pressed),
                    'k' | 'l' => self.lightness(c == 'l', pressed),
                    'a' => self.show_about = true,
                    _ => {}
                }
                // Remember to add new keys to keyboard_help_window !
            }
            _ => (),
        }
    }

    fn kbd_zoom(&mut self, inwards: bool, active: bool) {
        if active {
            self.movement.zoom = if inwards {
                ZOOM_SPEED
            } else {
                1.0 / ZOOM_SPEED
            };
        } else if inwards {
            if self.movement.zoom > 1. {
                self.movement.zoom = 0.;
            }
        } else if self.movement.zoom < 1. {
            /* !active !inwards */
            self.movement.zoom = 0.;
        }
    }

    fn expo(&mut self, increase: bool, active: bool) {
        if active {
            let magnitude = self.exponent.step();
            let sign = if increase { 1. } else { -1. };
            self.movement.exponent = sign * magnitude;
        } else {
            minimax(&mut self.movement.exponent, 0., 0., increase);
        }
    }

    fn palette(&mut self, increment: bool) {
        let delta = if increment { 1 } else { -1 };
        self.palette.colourer += delta;
    }

    field_fn!(gradient, offset, gamma, saturation, lightness);
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
