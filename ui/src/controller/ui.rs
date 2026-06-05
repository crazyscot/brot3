//! User interface entrypoint and motion control
//!
//! Portions of this file are based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

use std::sync::atomic::Ordering;

use brot3_lib::{
    data::{Algorithm, NumericType, Palette, PushExponent},
    engine,
};
use easy_cast::{Cast as _, CastApprox as _, CastFloat as _};
use easy_shader_runner::{UiState, egui};

use super::{DVec2, Instant};

impl super::Controller {
    pub(super) fn perturb_implemented(&self) -> bool {
        self.state.algorithm == Algorithm::Mandelbrot && self.state.exponent.is_two()
    }

    pub(super) fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.render_pass = self.render_pass.wrapping_add(1);
        self.service_channels();
        self.shader_last_elapsed = ui_state.last_elapsed;

        if self.perturbation_mode && !self.perturb_implemented() {
            let _ = egui::Window::new("Unimplemented")
                    .collapsible(false)
                    .resizable(false)
                    .fixed_pos(egui::pos2(
                        (self.state.viewport_size.x / 2).cast(),
                        (self.state.viewport_size.y / 2).cast(),
                    ))
                    .frame(egui::Frame::window(&ctx.global_style()).fill(egui::Color32::DARK_RED).inner_margin(10.0))
                    .show(ctx, |ui| {
                        let _ = ui.label(
                        "Perturbation mode is not yet implemented here. Only Mandelbrot at power 2 is currently supported.",
                    );
                });
        }

        let task = self.loading_task.as_ref();
        if task.is_some_and(tokio::task::JoinHandle::is_finished) {
            use futures::FutureExt as _;
            let task = self.loading_task.take().unwrap();
            match task.now_or_never() {
                Some(Err(e)) => log::warn!("task join error: {e}"),
                Some(Ok(None)) => (),
                Some(Ok(Some(s))) => {
                    log::debug!("Loaded state: {s:#?}");
                    self.state.merge(s);
                    self.just_loaded();
                }
                None => unreachable!(),
            }
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
            Self::fps_window(
                ctx,
                *ui_state,
                self.shader_last_elapsed.filter(|_| self.show_timings),
            );
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
            self.save_image_ui(ctx);
        }
        if self.show_save_position {
            self.save_position_ui(ctx);
        }
        if self.show_open {
            self.open_ui(ctx);
        }
        if self.save_busy.load(Ordering::Relaxed) {
            Self::save_busy_window(ctx);
        }
        self.error_modal(ctx);

        self.fullscreen_checkbox = ui_state.fullscreen_active;

        // Don't action initial-fullscreen requests on the first four passes. They get lost.
        if let Some(_s) = self.fullscreen_requested
            && self.state.viewport_size.y != 0
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
        let factor32: f32 = factor.cast_approx();
        let zoom2 = self.movement.zoom2;
        if zoom2 != 1.0 {
            let zoom_in = zoom2.is_sign_positive();
            let raw_zoom = zoom2.abs();
            assert!(raw_zoom >= 1.0);
            let dfactor = (raw_zoom - 1.0) * factor * dt + 1.0;
            let new_zoom = if zoom_in {
                self.state.viewport_zoom.0 * dfactor
            } else {
                self.state.viewport_zoom.0 / dfactor
            };
            self.update_zoom_factor(new_zoom);
            self.reiterate = true;
        }
        let movement = &mut self.movement;
        if movement.translate != DVec2::ZERO {
            self.state.viewport_translate +=
                movement.translate * factor / self.state.viewport_zoom.0 * dt;
            self.reiterate = true;
        }
        if movement.exponent != 0. {
            let current = match self.state.exponent.typ {
                NumericType::Integer => self.state.exponent.int.cast(),
                NumericType::Float => self.state.exponent.real,
            };
            let new_exp = (current + factor32 * movement.exponent)
                .clamp(PushExponent::MIN, PushExponent::MAX);
            if self.state.exponent.real != new_exp {
                self.reiterate = true;
                self.state.exponent.real = new_exp;
                if self.state.exponent.typ == NumericType::Integer {
                    self.state.exponent.int = self.state.exponent.real.cast_nearest();
                }
            }
            movement.exponent = 0.;
        }

        macro_rules! palette_fields {
            ($($id:ident), *) => {
                $(
                    if movement.$id != 0. {
                        self.state.palette.$id = (self.state.palette.$id + factor32 * movement.$id).clamp(Palette::MINIMA.$id, Palette::MAXIMA.$id);
                        movement.$id = 0.;
                    }
                )*
            }
        }
        palette_fields!(gradient, offset);
    }

    pub(crate) fn modifier_key_factor(&self) -> f64 {
        use super::keyboard::ModifiersExt as _;
        const SHIFT_FACTOR: f64 = core::f64::consts::E;
        const ALT_FACTOR: f64 = 1.0 / SHIFT_FACTOR;
        let mut factor = match (
            self.keyboard_modifiers.shift(),
            self.keyboard_modifiers.alt(),
        ) {
            (true, false) => SHIFT_FACTOR,
            (false, true) => ALT_FACTOR,
            (_, _) => 1.0,
        };
        if self.keyboard_modifiers.ctrl() {
            factor *= factor;
        }
        factor
    }

    fn recompute_perturbation(&mut self, graphics_context: &easy_shader_runner::GraphicsContext) {
        let mut dest = Vec::with_capacity(crate::MAX_MAX_ITERATIONS.cast());

        let start = Instant::now();
        engine::mandelbrot_perturbed_compute_reference_iters(
            &mut dest,
            &self.state.viewport_translate,
            self.state.algorithm,
            self.state.max_iter,
        );
        graphics_context.queue.write_buffer(
            self.perturbation.buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&dest),
        );
        self.perturbation.points = std::mem::take(&mut dest);
        self.last_perturb_time = start.elapsed();
    }
}
