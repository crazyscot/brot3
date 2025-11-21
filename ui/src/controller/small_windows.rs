// (c) 2025 Ross Younger

use super::DVec2;
use easy_shader_runner::{egui, UiState};

impl super::Controller {
    pub(crate) fn scale_bar(&mut self, ctx: &egui::Context) {
        use egui::epaint::{self, Color32};

        // Don't render this on the first pass before we know the window size. That gives it a bad default position.
        if self.size.y == 0 {
            return;
        }
        // Bottom centre of window
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

    pub(crate) fn fps_window(&mut self, ctx: &egui::Context, ui_state: &UiState) {
        egui::Window::new("fps")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10., -10.))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps()));
            });
    }

    pub(crate) fn context_menu_window(&mut self, ctx: &egui::Context, pos: DVec2) {
        let scale = ctx.pixels_per_point();
        let r = egui::Window::new("right_click_menu")
            .frame(egui::Frame::NONE)
            .title_bar(false)
            .resizable(false)
            .fixed_pos([pos.x as f32 / scale, pos.y as f32 / scale])
            .show(ctx, |ui| {
                if ui.button("Inspector...").clicked() {
                    self.inspector.position = self.pixel_address_to_complex(pos);
                    self.inspector.active = true;
                    self.context_menu = None;
                    self.inspector.stale = true;
                    self.show_coords_window = true;
                }
            });
        if let Some(r) = r
            && r.response.clicked_elsewhere()
        {
            //eprintln!("{r:?}");
            self.context_menu = None;
        }
    }
}
