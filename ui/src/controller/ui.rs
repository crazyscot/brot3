use easy_shader_runner::{UiState, egui};
use egui::epaint;

use super::{DVec2, Instant};
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
        egui_extras::install_image_loaders(ctx);
        ui_state.vsync = self.vsync;
        ui_state.fullscreen = self.fullscreen_requested;
        self.apply_movement();

        if self.show_ui {
            self.controls_window(ctx);

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
        }
        self.resized = false;
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
                        self.palette.$id = (self.palette.$id + movement.$id).clamp(shader_common::Palette::MINIMA.$id, shader_common::Palette::MAXIMA.$id);
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

                egui::CollapsingHeader::new("Exponent").show(ui, |ui| {
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
                });

                ui.label(egui::RichText::new("Iterations"));
                if ui
                    .add(egui::Slider::new(&mut self.max_iter, 1..=100_000).logarithmic(true))
                    .changed()
                {
                    self.reiterate = true;
                }

                ui.checkbox(&mut self.fractional_iters, "Fractional iterations");

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
                egui::CollapsingHeader::new("Palette controls")
                    .id_salt("palette-detail")
                    .show(ui, |ui| {
                        macro_rules! palette_slider {
                            ($($id:ident), * ) => {
                                $(
                                    ui.add(egui::Slider::new(&mut self.palette.$id, shader_common::Palette::MINIMA.$id ..= shader_common::Palette::MAXIMA.$id));
                                )*
                            };
                        }
                        // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
                        ui.label(egui::RichText::new("Gradient"));
                        palette_slider!(gradient);
                        ui.label(egui::RichText::new("Offset"));
                        palette_slider!(offset);
                        // Hide parameters when they don't apply
                        match self.palette.colourer {
                            Colourer::LogRainbow | Colourer::SqrtRainbow => {
                                ui.label(egui::RichText::new("Saturation"));
                                palette_slider!(saturation);
                                ui.label(egui::RichText::new("Lightness"));
                                palette_slider!(lightness);
                            }
                            Colourer::Monochrome => {
                                ui.label(egui::RichText::new("Gamma"));
                                palette_slider!(gamma);
                            }
                            _ => (),
                        }
                    });

                ui.separator();

                ui.checkbox(&mut self.show_coords_window, "Show co-ordinates");
                ui.checkbox(&mut self.show_scale_bar, "Scale bar");
                ui.checkbox(&mut self.keyboard_help, "Keyboard help");
                ui.checkbox(&mut self.show_fps, "Show FPS");
                ui.checkbox(&mut self.vsync, "vsync");
                ui.checkbox(&mut self.fullscreen_requested, "Fullscreen");

                if ui.button("About").clicked() {
                    self.show_about = true;
                }
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

    fn scale_bar(&mut self, ctx: &egui::Context) {
        use epaint::Color32;

        // Don't render this on the first pass before we know the window size. That gives it a bad default position.
        if self.size.y == 0 {
            return;
        }
        let pos = ((self.size.x / 2) as f32, (self.size.y - 10) as f32);

        let mut bar = egui::Area::new(egui::Id::new("scalebar"))
            .pivot(egui::Align2::CENTER_BOTTOM)
            .default_pos(pos);
        if self.resized {
            bar = bar.current_pos(pos);
        }
        bar.show(ctx, |ui| {
            // Within this anchoring, lay out left to right:
            egui::Grid::new("scalebar_inner").show(ui, |ui| {
                // The bar is a line, 100 pixels long, dashed black & white
                // White segment is 10 pixels wide to provide a contrasting of sorts
                const SCALE_BAR_SIZE: f32 = 100.;
                const SCALE_BAR_WIDTH: f32 = 10.;
                let (resp, painter) = ui.allocate_painter(
                    (SCALE_BAR_SIZE, SCALE_BAR_WIDTH).into(),
                    egui::Sense::empty(),
                );
                let points = [
                    resp.rect.min + egui::vec2(0., SCALE_BAR_WIDTH / 2.),
                    resp.rect.min + egui::vec2(SCALE_BAR_SIZE, SCALE_BAR_WIDTH / 2.),
                ];
                let shape1 = epaint::Shape::LineSegment {
                    points,
                    stroke: epaint::Stroke::new(SCALE_BAR_WIDTH, Color32::WHITE),
                };
                painter.add(shape1);
                let shape2 = epaint::Shape::dashed_line(
                    &points,
                    epaint::Stroke::new(SCALE_BAR_WIDTH - 2., Color32::BLACK),
                    SCALE_BAR_WIDTH,
                    SCALE_BAR_WIDTH,
                );
                painter.add(shape2);
                let bar_mid = (resp.rect.max.y + resp.rect.min.y) / 2.;
                let window_pos: egui::Pos2 = (resp.rect.max.x + SCALE_BAR_WIDTH, bar_mid).into();
                let pix_c = self.pixel_complex_size() * ui.pixels_per_point() as f64;
                let pixel_legend = pix_c * SCALE_BAR_SIZE as f64;
                egui::Window::new("scale label")
                    .title_bar(false)
                    .resizable(false)
                    .interactable(false)
                    .pivot(egui::Align2::LEFT_CENTER)
                    .fixed_pos(window_pos)
                    .show(ctx, |ui| {
                        ui.label(egui::RichText::new(format!("{pixel_legend:.3e}",)));
                    });
            });
        });
    }

    fn fps_window(&mut self, ctx: &egui::Context, ui_state: &UiState) {
        egui::Window::new("fps")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10., -10.))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps()));
            });
    }

    fn about_modal(&mut self, ctx: &egui::Context) {
        if egui::Modal::new("about".into())
            .show(ctx, |ui| {
                use crate::build_info;
                ui.label(egui::RichText::new("About brot3").size(18.));
                ui.label(
                    egui::RichText::new(format!("Version: {}", build_info::PKG_VERSION)).italics(),
                );
                if let Some(ver) = build_info::GIT_VERSION {
                    let dirty = if build_info::GIT_DIRTY.unwrap_or(false) {
                        "-dirty"
                    } else {
                        ""
                    };
                    ui.label(egui::RichText::new(format!("git version: {ver}{dirty}")).italics());
                }

                ui.add_space(12.);
                ui.image(egui::include_image!("../../../icons/original,origin=-1.259742+0.377104i,axes=0.01+0.01i,max=512,col=lch-gradient.png"));
                ui.add_space(6.);

                ui.label(
                    egui::RichText::new("Dedicated to the memory of Benoît B. Mandelbrot.")
                );
                ui.add_space(12.);
                if ui.button("License").clicked() {
                    self.show_license = true;
                }
            })
            .should_close()
        {
            self.show_about = false;
        }
    }

    fn license_modal(&mut self, ctx: &egui::Context) {
        if egui::Modal::new("license".into())
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("brot3 terms of use").size(18.));
                ui.label(egui::RichText::new("The MIT License").size(12.));
                ui.label(
                    r"
Copyright (C) 2025 Ross Younger

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
            ",
                );
            })
            .should_close()
        {
            self.show_license = false;
        }
    }
}
