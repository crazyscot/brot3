//! Minor windows
// (c) 2025 Ross Younger

use std::{path::Path, sync::Arc};

use easy_shader_runner::{UiState, egui};
use rfd::AsyncFileDialog;

use super::DVec2;

#[allow(unused_results)]
impl super::Controller {
    pub(crate) fn scale_bar(&mut self, ctx: &egui::Context) {
        use egui::epaint::{self, Color32};

        // Don't render this on the first pass before we know the window size. That gives it a bad
        // default position.
        if self.size.y == 0 {
            return;
        }
        // Bottom centre of window
        #[allow(clippy::cast_precision_loss)]
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
                let bar_mid = f32::midpoint(resp.rect.max.y, resp.rect.min.y);
                let window_pos: egui::Pos2 = (resp.rect.max.x + SCALE_BAR_WIDTH, bar_mid).into();
                let pix_c = self.pixel_complex_size() * f64::from(ui.pixels_per_point());
                let pixel_legend = pix_c * f64::from(SCALE_BAR_SIZE);
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

    pub(crate) fn fps_window(ctx: &egui::Context, ui_state: &UiState) {
        egui::Window::new("fps")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10., -10.))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps()));
            });
    }

    #[allow(clippy::cast_possible_truncation)]
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

    pub(crate) fn save_image_ui(&mut self, _ctx: &egui::Context) -> Result<(), anyhow::Error> {
        self.show_save = false;
        if !*self
            .save_active
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock save_active"))?
        {
            *self
                .save_active
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock save_active"))? = true;

            let default_filename = format!(
                "brot3_{datetime}_{description}.png",
                datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
                description = self.fragment_constants(false).display_string(),
            );

            let last_save_dir = {
                let guard = self
                    .last_save_dir
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock last_save_dir"))?;
                (*guard).clone()
            };

            let default_save_dir: Box<dyn AsRef<Path>> = match last_save_dir {
                Some(dir) if dir.is_dir() => Box::new(dir),
                _ => {
                    let fallback = dirs::picture_dir()
                        .or_else(dirs::desktop_dir)
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    Box::new(fallback)
                }
            };

            let task = AsyncFileDialog::new()
                .add_filter("PNG image", &["png"])
                .set_title("Save image")
                .set_directory(default_save_dir.as_ref())
                .set_file_name(&default_filename)
                .save_file();
            let save_active = Arc::clone(&self.save_active);
            let save_dir = Arc::clone(&self.last_save_dir);
            let error_message_buffer = Arc::clone(&self.error_message);
            let perturbation_points = self.perturbation.points.clone();
            let consts = self.fragment_constants(true);
            tokio::spawn(async move {
                if let Some(file) = task.await {
                    let filename = file.path().to_owned();
                    match crate::save::do_save_image(&filename, consts, &perturbation_points) {
                        Ok(()) => {
                            let parent = filename
                                .parent()
                                .unwrap_or_else(|| Path::new("."))
                                .to_path_buf();
                            *save_dir.lock().unwrap() = Some(parent);
                        }
                        Err(e) => {
                            eprintln!("Error saving image: {e}");
                            *error_message_buffer.lock().unwrap() =
                                Some(format!("Failed to save image: {e}"));
                        }
                    }
                } // else it was cancelled
                *save_active.lock().unwrap() = false;
            });
        }
        Ok(())
    }

    pub(crate) fn error_modal(&mut self, ctx: &egui::Context) {
        if let Some(message) = self.error_message.lock().map_or_else(
            |e| Some(format!("Failed to lock error_message: {e}")),
            |guard| guard.clone(),
        ) {
            let _ = egui::Modal::new("error".into()).show(ctx, |ui| {
                ui.label(egui::RichText::new("Error").size(18.));
                ui.add_space(12.);
                ui.label(message);
                ui.add_space(12.);
                if ui.button("OK").clicked() {
                    self.clear_error_modal();
                }
            });
        }
    }

    pub(crate) fn clear_error_modal(&mut self) {
        if let Ok(mut guard) = self.error_message.lock() {
            *guard = None;
        }
    }
}
