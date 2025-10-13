use easy_shader_runner::{UiState, egui};

use super::{DVec2, Instant};
use crate::controller::egui::Separator;
use shader_common::{Algorithm, Colourer};

pub(crate) const DEFAULT_WIDTH: f32 = 130.;

const EXPONENT_MIN: f32 = 0.;
const EXPONENT_MIN_INT: u32 = 0;
const EXPONENT_MAX: f32 = 20.;
const EXPONENT_MAX_INT: u32 = 20;

impl super::Controller {
    pub(super) fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        _graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        ui_state.vsync = self.vsync;
        self.apply_movement();

        if self.show_ui {
            self.controls_window(ctx);

            if self.show_coords_window {
                self.coords_window(ctx);
            }
            if self.show_fps {
                self.fps_window(ctx, ui_state);
            }
            if self.keyboard_help {
                self.keyboard_help_window(ctx);
            }
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
        if movement.exponent != 0. {
            let new_exp =
                (self.exponent.value + movement.exponent).clamp(EXPONENT_MIN, EXPONENT_MAX);
            if self.exponent.value != new_exp {
                self.reiterate = true;
                self.exponent.value = new_exp;
                if self.exponent.is_integer {
                    self.exponent.value_i = self.exponent.value.round() as u32;
                }
            }
            movement.exponent = 0.;
        }

        macro_rules! palette_fields {
            ($($id:ident), * ) => {
                $(
                    if movement.$id != 0. {
                        self.palette.$id += movement.$id;
                        movement.$id = 0.;
                    }
                )*
            }
        }
        palette_fields!(gradient, offset, gamma, saturation, lightness);
    }

    fn controls_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("brot3")
            .default_width(DEFAULT_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                use shader_common::NumericType;

                let algorithm_before = self.algorithm;
                egui::ComboBox::from_label(egui::RichText::new("Fractal"))
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
                ui.add(Separator::default().shrink(10.));

                ui.label(egui::RichText::new("Exponent").size(15.0));
                egui::Grid::new("exponent_grid").show(ui, |ui| {
                    let previous_int = self.exponent.is_integer;
                    ui.radio_value(&mut self.exponent.is_integer, true, "Integer");
                    ui.radio_value(&mut self.exponent.is_integer, false, "Float");
                    ui.end_row();
                    if self.exponent.is_integer != previous_int {
                        if previous_int {
                            // was integer, now float
                            self.exponent.value = self.exponent.value_i as f32;
                        } else {
                            // was float, now integer
                            self.exponent.value_i = self.exponent.value.round() as u32;
                            self.exponent.value = self.exponent.value_i as f32;
                        }
                        self.reiterate = true;
                    }
                });
                match self.exponent.variant() {
                    NumericType::Integer => {
                        if ui
                            .add(egui::Slider::new(
                                &mut self.exponent.value_i,
                                EXPONENT_MIN_INT..=EXPONENT_MAX_INT,
                            ))
                            .changed()
                        {
                            self.exponent.value = self.exponent.value_i as f32;
                            self.reiterate = true;
                        }
                    }
                    NumericType::Float => {
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut self.exponent.value,
                                    EXPONENT_MIN..=EXPONENT_MAX,
                                )
                                .step_by(0.1),
                            )
                            .changed()
                        {
                            self.reiterate = true;
                        }
                    }
                }

                if ui
                    .add(egui::Checkbox::new(
                        &mut self.exponent.is_negative,
                        "Negative",
                    ))
                    .changed()
                {
                    self.reiterate = true;
                }

                ui.add(Separator::default().shrink(10.));

                ui.label(egui::RichText::new("Iterations"));
                if ui
                    .add(egui::Slider::new(&mut self.max_iter, 1..=100_000).logarithmic(true))
                    .changed()
                {
                    self.reiterate = true;
                }
                ui.separator();

                egui::ComboBox::from_label("Palette")
                    .selected_text(format!("{:?}", self.palette.colourer))
                    .show_ui(ui, |ui| {
                        use strum::IntoEnumIterator as _;
                        for it in Colourer::iter() {
                            let label: &'static str = it.into();
                            ui.selectable_value(&mut self.palette.colourer, it, label);
                        }
                    });
                // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
                ui.label(egui::RichText::new("Gradient"));
                ui.add(egui::Slider::new(&mut self.palette.gradient, 0.1..=10.));
                ui.label(egui::RichText::new("Offset"));
                ui.add(egui::Slider::new(&mut self.palette.offset, -10.0..=10.));
                // Hide parameters when they don't apply
                match self.palette.colourer {
                    Colourer::LogRainbow | Colourer::SqrtRainbow => {
                        ui.label(egui::RichText::new("Saturation"));
                        ui.add(egui::Slider::new(&mut self.palette.saturation, 0. ..=100.));
                        ui.label(egui::RichText::new("Lightness"));
                        ui.add(egui::Slider::new(&mut self.palette.lightness, 0. ..=100.));
                    }
                    Colourer::Monochrome => {
                        ui.label(egui::RichText::new("Gamma"));
                        ui.add(egui::Slider::new(&mut self.palette.gamma, 0.0..=4.0));
                    }
                    _ => (),
                }
                ui.separator();

                ui.checkbox(&mut self.show_coords_window, "Show co-ordinates");
                ui.checkbox(&mut self.keyboard_help, "Keyboard help");
                ui.checkbox(&mut self.show_fps, "Show FPS");
                ui.checkbox(&mut self.vsync, "vsync");
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
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::splat(-10.0))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps()));
            });
    }
}
