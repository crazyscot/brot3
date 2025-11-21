//! Co-ordinates readout, inspector and wrangling
// (c) 2025 Ross Younger

use easy_shader_runner::egui;
use shader_common::FragmentConstants;
use util::dynfmt;

impl super::Controller {
    /// Calculates the decimal precision required to satisfactorily express a fractal part co-ordinate.
    ///
    /// We need two guard digits to correctly reconstruct to desired accuracy.
    /// <http://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html#693> refers.
    fn precision_digits(&self) -> usize {
        let pixel_size = self.pixel_complex_size();
        ((0.0 - pixel_size.log10()).ceil() + 2.0) as usize
    }

    pub(crate) fn coords_window(&mut self, ctx: &egui::Context) {
        let precision = self.precision_digits();
        // Don't render this on the first pass before we know the window size. That gives it a bad default position.
        if self.size.y == 0 {
            return;
        }
        // Top right of window
        let pos = ((self.size.x - 10) as f32, 10.0);

        egui::Window::new("coords")
            .title_bar(false)
            .resizable(false)
            .pivot(egui::Align2::RIGHT_TOP)
            .default_pos(pos)
            .show(ctx, |ui| {
                ui.set_width(10.); // hack: ensures the separator doesn't inflate the window
                egui::Grid::new("coords_position").show(ui, |ui| {
                    ui.label(egui::RichText::new("Centre").italics());
                    ui.end_row();
                    ui.label("Fractal X (Re)");
                    ui.monospace(dynfmt!(
                        self.viewport_translate.x.to_f64().value(),
                        precision
                    ));
                    ui.end_row();
                    ui.label("Fractal Y (Im)");
                    ui.monospace(dynfmt!(
                        self.viewport_translate.y.to_f64().value(),
                        precision
                    ));
                    ui.end_row();
                    ui.label("Zoom");
                    let zoom = self.viewport_zoom * f64::from(FragmentConstants::UI_ZOOM_FACTOR);
                    if zoom < 1000. {
                        ui.monospace(format!("{zoom:.2}"));
                    } else if zoom < 10_000_000. {
                        ui.monospace(format!("{zoom:.1}"));
                    } else {
                        ui.monospace(format!("{zoom:+.2e}"));
                    }
                });

                if self.inspector.active {
                    ui.separator();
                    ui.label(egui::RichText::new("Marked position").italics());
                    egui::Grid::new("inspect_position").show(ui, |ui| {
                        let complex_pos = &self.inspector.position;
                        ui.label("X (Re)");
                        ui.monospace(dynfmt!(complex_pos.x.to_f64().value(), precision));
                        ui.end_row();
                        ui.label("Y (Im)");
                        ui.monospace(dynfmt!(complex_pos.y.to_f64().value(), precision));
                        ui.end_row();
                        let inside: bool = self.inspector.data.inside();
                        ui.label("Iterations");
                        if inside {
                            ui.monospace("∞");
                        } else {
                            // We only need to report in standard precision for iterations
                            ui.monospace(dynfmt!(
                                self.inspector.data.iters(self.palette.colour_style),
                                6
                            ));
                        }
                        ui.end_row();
                        ui.label("Distance");
                        ui.monospace(dynfmt!(self.inspector.data.distance()));
                        ui.end_row();
                        ui.label("Final angle");
                        ui.monospace(dynfmt!(self.inspector.data.angle()));
                        ui.end_row();
                        ui.label("Final radius");
                        ui.monospace(dynfmt!(self.inspector.data.radius_sqr().sqrt()));
                        ui.end_row();
                    });
                    if ui.button("Close inspector").clicked() {
                        self.inspector.active = false;
                    }
                }
            });
    }

    pub(crate) fn mouse_on_marker(&self) -> bool {
        use shader_common::INSPECTOR_MARKER_SIZE;
        self.inspector.active
            && self
                .mouse_position
                .distance_squared(self.complex_point_to_pixel(&self.inspector.position))
                < INSPECTOR_MARKER_SIZE as f64 * INSPECTOR_MARKER_SIZE as f64
    }

    pub(crate) fn set_mouse_pointer(&mut self, ctx: &egui::Context) {
        if self.inspector.dragging {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if self.mouse_on_marker() {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }
    }
    pub(crate) fn update_inspector(&mut self) {
        self.inspector.stale = false;
        let consts = self.fragment_constants(false);
        self.inspector.data = shader::fractal::render(&consts, self.inspector.position.as_vec2());
    }
}
