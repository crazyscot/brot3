use easy_shader_runner::{UiState, egui};
use shader::enums::Algorithm;
use util::NumericType;

use super::{DVec2, Instant};

impl super::Controller {
    #[allow(clippy::cast_precision_loss)]
    pub(super) const EXPONENT_MAX: f32 = Self::EXPONENT_MAX_INT as f32;
    pub(super) const EXPONENT_MAX_INT: i32 = 20;
    #[allow(clippy::cast_precision_loss)]
    pub(super) const EXPONENT_MIN: f32 = Self::EXPONENT_MIN_INT as f32;
    pub(super) const EXPONENT_MIN_INT: i32 = 0;

    pub(super) fn perturb_implemented(&self) -> bool {
        self.algorithm == Algorithm::Mandelbrot && self.exponent.is_two()
    }

    pub(super) fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.render_pass += 1;

        if self.perturbation_mode && !self.perturb_implemented() {
            #[allow(clippy::cast_precision_loss)]
                let _ = egui::Window::new("Unimplemented")
                    .collapsible(false)
                    .resizable(false)
                    .fixed_pos(egui::pos2(
                        self.size.x as f32 / 2.0,
                        self.size.y as f32 / 2.0,
                    ))
                    .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::DARK_RED).inner_margin(10.0))
                    .show(ctx, |ui| {
                        let _ = ui.label(
                        "Perturbation mode is not yet implemented here. Only Mandelbrot at power 2 is currently supported.",
                    );
                });
        }

        egui_extras::install_image_loaders(ctx);
        ui_state.vsync = self.vsync;
        self.apply_movement();

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
            Self::fps_window(ctx, ui_state);
        }
        if self.keyboard_help {
            Self::keyboard_help_window(ctx);
        }
        if self.show_about {
            self.about_modal(ctx);
        }
        if self.show_license {
            self.license_modal(ctx);
        }
        if self.show_save {
            let _ = self.save_image_ui(ctx);
        }
        self.error_modal(ctx);

        self.fullscreen_checkbox = ui_state.fullscreen_active;

        // Don't action initial-fullscreen requests on the first four passes. They get lost.
        if let Some(_s) = self.fullscreen_requested
            && self.size.y != 0
            && self.render_pass > 4
        {
            ui_state.fullscreen_requested = self.fullscreen_requested;
            self.fullscreen_requested = None;
        }

        self.resized = false;
        self.set_mouse_pointer(ctx);
        if (self.reiterate || self.always_reiterate) && self.perturbation_mode {
            self.recompute_perturbation(graphics_context);
        }
        if self.inspector.active && self.inspector.stale {
            self.update_inspector();
        }
    }

    #[allow(clippy::float_cmp)]
    fn apply_movement(&mut self) {
        let dt = self.last_instant.elapsed().as_secs_f64();
        self.last_instant = Instant::now();
        let factor = self.modifier_key_factor();
        #[allow(clippy::cast_possible_truncation)]
        let factor32 = factor as f32;
        let zoom2 = self.movement.zoom2;
        if zoom2 != 1.0 {
            let zoom_in = zoom2.is_sign_positive();
            let raw_zoom = zoom2.abs();
            assert!(raw_zoom >= 1.0);
            let dfactor = (raw_zoom - 1.0) * factor * dt + 1.0;
            let new_zoom = if zoom_in {
                self.viewport_zoom.0 * dfactor
            } else {
                self.viewport_zoom.0 / dfactor
            };
            self.update_zoom_factor(new_zoom);
            self.reiterate = true;
        }
        let movement = &mut self.movement;
        if movement.translate != DVec2::ZERO {
            self.viewport_translate += movement.translate * factor / self.viewport_zoom.0 * dt;
            self.reiterate = true;
        }
        if movement.exponent != 0. {
            let new_exp = (self.exponent.real + factor32 * movement.exponent)
                .clamp(Self::EXPONENT_MIN, Self::EXPONENT_MAX);
            if self.exponent.real != new_exp {
                self.reiterate = true;
                self.exponent.real = new_exp;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                if self.exponent.typ == NumericType::Integer {
                    self.exponent.int = self.exponent.real.round() as i32;
                }
            }
            movement.exponent = 0.;
        }
        if movement.exponent_im != 0. {
            let new_exp = (self.exponent.imag + factor32 * movement.exponent_im)
                .clamp(Self::EXPONENT_MIN, Self::EXPONENT_MAX);
            if self.exponent.imag != new_exp && self.exponent.typ != NumericType::Integer {
                self.reiterate = true;
                self.exponent.imag = new_exp;
            }
            movement.exponent_im = 0.;
        }

        macro_rules! palette_fields {
            ($($id:ident), *) => {
                $(
                    if movement.$id != 0. {
                        self.palette.$id = (self.palette.$id + factor32 * movement.$id).clamp(shader::push_constants::Palette::MINIMA.$id, shader::push_constants::Palette::MAXIMA.$id);
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

    fn recompute_perturbation(&mut self, graphics_context: &easy_shader_runner::GraphicsContext) {
        shader::fractal::mandelbrot_perturbed_compute_reference_iters(
            &mut self.perturbation.points,
            &self.viewport_translate,
            self.algorithm,
            self.max_iter,
        );

        graphics_context.queue.write_buffer(
            self.perturbation.buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&self.perturbation.points),
        );
    }
}
