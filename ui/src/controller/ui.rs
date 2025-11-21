use easy_shader_runner::{egui, UiState};

use super::{DVec2, Instant};

impl super::Controller {
    pub(super) const EXPONENT_MIN: f32 = 0.;
    pub(super) const EXPONENT_MIN_INT: u32 = 0;
    pub(super) const EXPONENT_MAX: f32 = 20.;
    pub(super) const EXPONENT_MAX_INT: u32 = 20;

    pub(super) fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        _graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        egui_extras::install_image_loaders(ctx);
        ui_state.vsync = self.vsync;
        ui_state.fullscreen = self.fullscreen_requested;
        self.apply_movement();
        if self.inspector.stale {
            self.update_inspector();
        }

        self.main_menu(ctx);

        if let Some(pos) = self.context_menu {
            self.context_menu_window(ctx, pos);
        }
        if self.show_controls {
            self.controls_window(ctx);
        }
        if self.show_coords_window {
            self.coords_window(ctx);
        }
        if self.show_scale_bar {
            self.scale_bar(ctx);
        }
        if self.show_fps {
            self.fps_window(ctx, ui_state);
        }
        if self.keyboard_help {
            self.keyboard_help_window(ctx);
        }
        if self.show_about {
            self.about_modal(ctx);
        }
        if self.show_license {
            self.license_modal(ctx);
        }

        self.resized = false;
        self.set_mouse_pointer(ctx);
    }

    fn apply_movement(&mut self) {
        let dt = self.last_instant.elapsed().as_secs_f64();
        self.last_instant = Instant::now();
        let factor = self.modifier_key_factor();
        let factor32 = factor as f32;
        let movement = &mut self.movement;
        if movement.zoom != 0.0 {
            self.viewport_zoom *= (movement.zoom - 1.0) * factor * dt + 1.0;
            self.reiterate = true;
        }
        if movement.translate != DVec2::ZERO {
            self.viewport_translate += movement.translate * factor / self.viewport_zoom * dt;
            self.reiterate = true;
        }
        if movement.exponent != 0. {
            let new_exp = (self.exponent.real + factor32 * movement.exponent)
                .clamp(Self::EXPONENT_MIN, Self::EXPONENT_MAX);
            if self.exponent.real != new_exp {
                self.reiterate = true;
                self.exponent.real = new_exp;
                if self.exponent.is_integer() {
                    self.exponent.int = self.exponent.real.round() as u32;
                }
            }
            movement.exponent = 0.;
        }
        if movement.exponent_im != 0. {
            let new_exp = (self.exponent.imag + factor32 * movement.exponent_im)
                .clamp(Self::EXPONENT_MIN, Self::EXPONENT_MAX);
            if self.exponent.imag != new_exp && !self.exponent.is_integer() {
                self.reiterate = true;
                self.exponent.imag = new_exp;
            }
            movement.exponent_im = 0.;
        }

        macro_rules! palette_fields {
            ($($id:ident), *) => {
                $(
                    if movement.$id != 0. {
                        self.palette.$id = (self.palette.$id + factor32 * movement.$id).clamp(shader_common::Palette::MINIMA.$id, shader_common::Palette::MAXIMA.$id);
                        movement.$id = 0.;
                    }
                )*
            }
        }
        palette_fields!(gradient, offset, gamma, saturation, lightness);
    }

    pub(crate) fn modifier_key_factor(&self) -> f64 {
        const SHIFT_FACTOR: f64 = core::f64::consts::E;
        const ALT_FACTOR: f64 = 1.0 / SHIFT_FACTOR;
        const CTRL_SHIFT: f64 = SHIFT_FACTOR * SHIFT_FACTOR;
        const CTRL_ALT: f64 = ALT_FACTOR * ALT_FACTOR;
        match (self.shift_pressed, self.alt_pressed, self.ctrl_pressed) {
            (true, false, false) => SHIFT_FACTOR,
            (true, false, true) => CTRL_SHIFT,
            (false, true, false) => ALT_FACTOR,
            (false, true, true) => CTRL_ALT,
            (_, _, _) => 1.0,
        }
    }
}
