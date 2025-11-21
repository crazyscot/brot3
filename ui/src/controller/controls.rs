//! Controls window
// (c) 2025 Ross Younger

use shader_common::enums::{Algorithm, ColourStyle, Colourer, Modifier};

use easy_shader_runner::egui;

impl super::Controller {
    pub(crate) const DEFAULT_WIDTH: f32 = 130.;

    pub(super) fn controls_window(&mut self, ctx: &egui::Context) {
        // Don't render this on the first pass before we know the window size. That gives it a bad default position.
        if self.size.y == 0 {
            return;
        }
        // Centre left of window
        let pos = (10.0, (self.size.y / 2) as f32);

        egui::Window::new("Controls")
            .default_width(Self::DEFAULT_WIDTH)
            .default_pos(pos)
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
                        let previous_typ = self.exponent.typ;
                        ui.radio_value(&mut self.exponent.typ, NumericType::Integer, "Integer");
                        ui.radio_value(&mut self.exponent.typ, NumericType::Float, "Float");
                        ui.radio_value(&mut self.exponent.typ, NumericType::Complex, "Complex");
                        ui.end_row();
                        match (previous_typ, self.exponent.typ) {
                            (NumericType::Integer, _) => {
                                self.exponent.real = self.exponent.int as f32;
                                self.exponent.imag = 0.0;
                            }
                            (_, NumericType::Integer) => {
                                self.exponent.int = self.exponent.real.round() as u32;
                                self.exponent.real = self.exponent.int as f32;
                                self.exponent.imag = 0.0;
                            }
                            (NumericType::Complex, NumericType::Float)  | (NumericType::Float, NumericType::Complex) => {
                                self.exponent.imag = 0.0;
                            }
                            (NumericType::Float, NumericType::Float) |  (NumericType::Complex, NumericType::Complex)=> (),
                            _ => todo!(),
                        }
                        if self.exponent.typ != previous_typ {
                            if previous_typ == NumericType::Integer {
                            } else {
                                // was float, now integer
                                self.exponent.int = self.exponent.real.round() as u32;
                                self.exponent.real = self.exponent.int as f32;
                            }
                            self.reiterate = true;
                        }
                    });
                    match self.exponent.variant() {
                        NumericType::Integer => {
                            if ui.add(egui::Slider::new(
                                    &mut self.exponent.int,
                                    Self::EXPONENT_MIN_INT..=Self::EXPONENT_MAX_INT,
                                ))
                                .changed()
                            {
                                self.exponent.real = self.exponent.int as f32;
                                self.reiterate = true;
                            }
                        }
                        NumericType::Float => {
                            if ui.add(
                                    egui::Slider::new(
                                        &mut self.exponent.real,
                                        Self::EXPONENT_MIN..=Self::EXPONENT_MAX,
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
                                        &mut self.exponent.real,
                                        Self::EXPONENT_MIN..=Self::EXPONENT_MAX,
                                    )
                                    .step_by(0.1),
                                )
                                .changed()
                            {
                                self.reiterate = true;
                            }
                        }
                        _ => todo!(),
                    }

                    if ui.add(egui::Checkbox::new(
                            &mut  self.exponent.real_is_negative,
                            "Negative",
                        ))
                        .changed()
                    {
                        self.reiterate = true;
                    }

                    if self.exponent.variant() == NumericType::Complex {
                        ui.label("Imaginary");
                            if ui.add(
                                    egui::Slider::new(
                                        &mut self.exponent.imag,
                                        Self::EXPONENT_MIN..=Self::EXPONENT_MAX,
                                    )
                                    .step_by(0.1),
                                )
                                .changed()
                            {
                                self.reiterate = true;
                            }
                            if ui.add(egui::Checkbox::new(
                                &mut self.exponent.imag_is_negative,
                                "Negative",
                            ))
                            .changed()
                        {
                            self.reiterate = true;
                        }
                    }
                });

                ui.label(egui::RichText::new("Max Iterations"));
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
                egui::CollapsingHeader::new("Palette controls")
                    .id_salt("palette-detail")
                    .show(ui, |ui| {
                        egui::ComboBox::from_label("Colour Style")
                            .selected_text(format!("{:?}", self.palette.colour_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in ColourStyle::iter() {
                                    let label: &'static str = it.into();
                                    ui.selectable_value(&mut self.palette.colour_style, it, label);
                                }
                            });
                        egui::ComboBox::from_label("Brightness Style")
                            .selected_text(format!("{:?}", self.palette.brightness_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in Modifier::iter() {
                                    let label: &'static str = it.into();
                                    ui.selectable_value(&mut self.palette.brightness_style, it, label);
                                }
                            });
                        egui::ComboBox::from_label("Saturation Style")
                            .selected_text(format!("{:?}", self.palette.saturation_style))
                            .show_ui(ui, |ui| {
                                use strum::IntoEnumIterator as _;
                                for it in Modifier::iter() {
                                    let label: &'static str = it.into();
                                    ui.selectable_value(&mut self.palette.saturation_style, it, label);
                                }
                            });

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
            })
            .unwrap();
    }
}
