use easy_shader_runner::{UiState, egui};

use super::{DVec2, Instant};

impl super::Controller {
    pub(super) fn ui_impl(
        &mut self,
        _ctx: &egui::Context,
        _ui_state: &mut UiState,
        _graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.apply_movement();
    }

    fn apply_movement(&mut self) {
        let dt = self.last_instant.elapsed().as_secs_f64();
        self.last_instant = Instant::now();
        let movement = &mut self.movement;
        if movement.zoom != 0.0 {
            self.viewport_zoom *= (movement.zoom - 1.0) * dt + 1.0;
            self.reiterate = true;
        }
        if movement.translate != DVec2::ZERO {
            self.viewport_translate += movement.translate / self.viewport_zoom * dt;
            self.reiterate = true;
        }
    }
}
