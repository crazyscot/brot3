//! Controls window
// (c) 2025 Ross Younger

use brot3_lib::data::{Algorithm, ColourStyle, Colourer, Modifier, NumericType, Palette};
use easy_shader_runner::egui;
use num_traits::AsPrimitive;
use strum::EnumMessage as _;

#[allow(unused_results)]
impl super::Controller {
    pub(crate) const DEFAULT_WIDTH: f32 = 130.;

    #[allow(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::missing_panics_doc
    )]
    pub(super) fn controls_window(&mut self, ctx: &egui::Context) {
        // Don't render this on the first pass before we know the window size. That gives it a bad
        // default position.
        if self.state.viewport_size.y == 0 {
            return;
        }
        // Centre left of window
        let pos = (10.0, (self.state.viewport_size.y / 2) as f32);

        egui::Window::new("Controls")
            .default_width(Self::DEFAULT_WIDTH)
            .default_pos(pos)
            .resizable(false)
            .show(ctx, |ui| {

                let algorithm_before = self.state.algorithm;
                egui::ComboBox::from_label(egui::RichText::new("Fractal"))
                    .selected_text(format!("{:?}", self.state.algorithm))
                    .show_ui(ui, |ui| {
                        use strum::IntoEnumIterator as _;
                        for it in Algorithm::iter() {
                            let label: &'static str = it.into();
                            ui.selectable_value(&mut self.state.algorithm, it, label)
                                .on_hover_text(it.get_documentation().unwrap_or_default());
                        }
                    });
                if self.state.algorithm != algorithm_before {
                    self.reiterate = true;
                }

                egui::CollapsingHeader::new("Exponent").show(ui, |ui| {
                    egui::Grid::new("exponent_grid").show(ui, |ui| {
                        let previous_typ = self.state.exponent.typ;
                        ui.radio_value(&mut self.state.exponent.typ, NumericType::Integer, "Integer");
                        ui.radio_value(&mut self.state.exponent.typ, NumericType::Float, "Float");
                        ui.radio_value(&mut self.state.exponent.typ, NumericType::Complex, "Complex");
                        ui.end_row();
                        match (previous_typ, self.state.exponent.typ) {
                            (NumericType::Integer, _) => {
                                self.state.exponent.real = self.state.exponent.int as f32;
                                self.state.exponent.imag = 0.0;
                            }
                            (_, NumericType::Integer) => {
                                self.state.exponent.int = self.state.exponent.real.round().as_();
                                self.state.exponent.real = self.state.exponent.int as f32;
                                self.state.exponent.imag = 0.0;
                            }
                            (NumericType::Complex, NumericType::Float)  | (NumericType::Float, NumericType::Complex) => {
                                self.state.exponent.imag = 0.0;
                            }
                            (NumericType::Float, NumericType::Float) |  (NumericType::Complex, NumericType::Complex)=> (),
                        }
                        if self.state.exponent.typ != previous_typ {
                            if previous_typ == NumericType::Integer {
                            } else {
                                // was float, now integer
                                self.state.exponent.int = self.state.exponent.real.round().as_();
                                self.state.exponent.real = self.state.exponent.int as f32;
                            }
                            self.reiterate = true;
                        }
                    });
                    match self.state.exponent.typ {
                        NumericType::Integer => {
                            if ui.add(egui::Slider::new(
                                    &mut self.state.exponent.int,
                                    -Self::EXPONENT_MAX_INT..=Self::EXPONENT_MAX_INT,
                                ))
                                .changed()
                            {
                                self.state.exponent.real = self.state.exponent.int as f32;
                                self.reiterate = true;
                            }
                        }
                        NumericType::Float => {
                            if ui.add(
                                    egui::Slider::new(
                                        &mut self.state.exponent.real,
                                        -Self::EXPONENT_MAX..=Self::EXPONENT_MAX,
                                    )
                                    .step_by(0.1),
                                )
                                .changed()
                            {
                                self.reiterate = true;
                            }
                        }
                        NumericType::Complex => {
                            ui.label("Real");
                            if ui.add(
                                    egui::Slider::new(
                                        &mut self.state.exponent.real,
                                        -Self::EXPONENT_MAX..=Self::EXPONENT_MAX,
                                    )
                                    .step_by(0.1),
                                )
                                .changed()
                            {
                                self.reiterate = true;
                            }
                        }
                    }

                    if self.state.exponent.typ == NumericType::Complex {
                        ui.label("Imaginary");
                        if ui.add(
                                egui::Slider::new(
                                    &mut self.state.exponent.imag,
                                    -Self::EXPONENT_MAX..=Self::EXPONENT_MAX,
                                )
                                .step_by(0.1),
                            )
                            .changed()
                        {
                            self.reiterate = true;
                        }
                    }
                });

                ui.label(egui::RichText::new("Max Iterations"));
                if ui
                    .add(egui::Slider::new(&mut self.state.max_iter, 1..=crate::MAX_MAX_ITERATIONS).logarithmic(true))
                    .changed()
                {
                    self.reiterate = true;
                }

                ui.separator();

                egui::ComboBox::from_label("Palette")
                    .selected_text(format!("{:?}", self.state.palette.colourer))
                    .show_ui(ui, |ui| {
                        use strum::IntoEnumIterator as _;
                        for it in Colourer::iter() {
                            let label: &'static str = it.into();
                            ui.selectable_value(&mut self.state.palette.colourer, it, label)
                                .on_hover_text(it.get_documentation().unwrap_or_default());
                        }
                    });
                egui::CollapsingHeader::new("Palette controls")
                    .id_salt("palette-detail")
                    .show(ui, |ui| {
                        egui::ComboBox::from_label("Colour Style")
                            .selected_text(format!("{:?}", self.state.palette.colour_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in ColourStyle::iter() {
                                    let label: &'static str = it.into();
                                    ui.selectable_value(&mut self.state.palette.colour_style, it, label)
                                        .on_hover_text(it.get_documentation().unwrap_or_default());
                                }
                            });
                        egui::ComboBox::from_label("Brightness Style")
                            .selected_text(format!("{:?}", self.state.palette.brightness_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in Modifier::iter() {
                                    let label: &'static str = it.into();
                                    if ui.selectable_value(&mut self.state.palette.brightness_style, it, label)
                                        .on_hover_text(it.get_documentation().unwrap_or_default())
                                        .clicked() {
                                            self.reiterate = true;
                                            self.inspector.stale = true;
                                        }
                                }
                            });
                        egui::ComboBox::from_label("Saturation Style")
                            .selected_text(format!("{:?}", self.state.palette.saturation_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in Modifier::iter() {
                                    let label: &'static str = it.into();
                                    if ui.selectable_value(&mut self.state.palette.saturation_style, it, label)
                                        .on_hover_text(it.get_documentation().unwrap_or_default()).clicked() {
                                            self.reiterate = true;
                                            self.inspector.stale = true;
                                        }
                                }
                            });

                        macro_rules! palette_slider {
                            ($($id:ident), * ) => {
                                $(
                                    ui.add(egui::Slider::new(&mut self.state.palette.$id, Palette::MINIMA.$id ..= Palette::MAXIMA.$id));
                                )*
                            };
                        }
                        // N.B. Each colourer is at liberty to scale gradient & offset as may be reasonable.
                        ui.label(egui::RichText::new("Gradient"));
                        palette_slider!(gradient);
                        ui.label(egui::RichText::new("Offset"));
                        palette_slider!(offset);
                        // Hide parameters when they don't apply
                        match self.state.palette.colourer {
                            Colourer::LogRainbow => {
                                ui.label(egui::RichText::new("Saturation"));
                                palette_slider!(saturation);
                                ui.label(egui::RichText::new("Lightness"));
                                palette_slider!(lightness);
                            }
                            Colourer::Monochrome | Colourer::IcyBlue => {
                                ui.label(egui::RichText::new("Gamma"));
                                palette_slider!(gamma);
                            }
                            _ => (),
                        }
                        if ui.checkbox(&mut self.state.iteration_cull, egui::RichText::new("Iteration cull")).clicked()
                        {
                            self.reiterate = true;
                        }
                    });
            })
            .unwrap();
    }
}
