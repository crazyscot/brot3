use easy_shader_runner::{UiState, egui};

use super::{DVec2, Instant};
use shader_common::Algorithm;

const DEFAULT_WIDTH: f32 = 130.;

impl super::Controller {
    pub(super) fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        _graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.apply_movement();

        self.controls_window(ctx);

        if self.show_coords_window {
            self.coords_window(ctx);
        }
        if self.show_fps {
            self.fps_window(ctx, ui_state);
        }
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

    fn controls_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("brot3")
            .default_width(DEFAULT_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                let algorithm_before = self.algorithm;
                egui::ComboBox::from_label(egui::RichText::new("Algorithm").size(15.0))
                    .selected_text(format!("{:?}", self.algorithm))
                    .show_ui(ui, |ui| {
                        use strum::IntoEnumIterator as _;
                        for it in Algorithm::iter() {
                            let label: &'static str = it.into();
                            ui.selectable_value(&mut self.algorithm, it, label);
                        }
                    });
                if self.algorithm != algorithm_before {
                    self.reiterate = true;
                }

                ui.separator();
                ui.label(egui::RichText::new("Iterations"));
                if ui
                    .add(egui::Slider::new(&mut self.max_iter, 1..=100_000).logarithmic(true))
                    .changed()
                {
                    self.reiterate = true;
                }
                ui.separator();

                ui.checkbox(&mut self.show_coords_window, "Show co-ordinates");
                ui.checkbox(&mut self.show_fps, "Show FPS");
            })
            .unwrap();
    }
    fn coords_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("coords")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10., 10.))
            .show(ctx, |ui| {
                egui::Grid::new("coords_position").show(ui, |ui| {
                    ui.label("Fractal X (Re)");
                    // TODO: What precision to show for deep zooms?
                    ui.monospace(format!(
                        "{:+.6e}",
                        self.viewport_translate.x.to_f32().value()
                    ));
                    ui.end_row();
                    ui.label("Fractal Y (Im)");
                    ui.monospace(format!(
                        "{:+.6e}",
                        self.viewport_translate.y.to_f32().value()
                    ));
                    ui.end_row();
                    ui.label("Zoom");
                    let zoom = self.viewport_zoom;
                    if zoom < 1000. {
                        ui.monospace(format!("{zoom:.2}"));
                    } else if zoom < 10_000_000. {
                        ui.monospace(format!("{zoom:.1}"));
                    } else {
                        ui.monospace(format!("{zoom:+.2e}"));
                    }
                    // ...
                });
            });
    }

    fn fps_window(&mut self, ctx: &egui::Context, ui_state: &UiState) {
        egui::Window::new("fps")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::splat(-10.0))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps()));
            });
    }
}
