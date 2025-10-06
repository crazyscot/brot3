use easy_shader_runner::winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

const MOVE_SPEED: f64 = 0.2;
const ZOOM_SPEED: f64 = 1.4;

impl super::Controller {
    pub(super) fn keyboard_input_impl(&mut self, key: KeyEvent) {
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
            Key::Character(c) => {
                let c = match c.chars().next() {
                    Some(ch) => ch,
                    None => return, // should never happen
                };
                match c {
                    'z' => self.kbd_zoom(true, pressed),
                    'x' => self.kbd_zoom(false, pressed),
                    'e' => self.expo(false, pressed),
                    'r' => self.expo(true, pressed),
                    _ => {}
                }
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
            /* !inwards */
            self.movement.zoom = 0.;
        }
    }

    fn expo(&mut self, increase: bool, active: bool) {
        if active {
            self.movement.exponent = if increase { 1 } else { -1 };
        } else if increase {
            if self.movement.exponent > 0 {
                self.movement.exponent = 0;
            }
        } else if self.movement.exponent < 0 {
            /* !inwards */
            self.movement.exponent = 0;
        }
    }
}
