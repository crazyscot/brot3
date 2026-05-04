//! Minor windows
// (c) 2025 Ross Younger

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use brot3_lib::{data::RenderMode, ui::UiState};
use easy_cast::{Cast as _, ConvApprox as _};
use easy_shader_runner::{UiState as EsrUiState, egui};
use rfd::FileDialog;
use tokio::task::JoinHandle;

use super::DVec2;
use crate::save::{self, LoadSaveError};

#[allow(unused_results)]
impl super::Controller {
    pub(crate) fn scale_bar(&mut self, ctx: &egui::Context) {
        use egui::epaint::{self, Color32};

        // Don't render this on the first pass before we know the window size. That gives it a bad
        // default position.
        if self.state.viewport_size.y == 0 {
            return;
        }
        // Bottom centre of window
        let pos = (
            (self.state.viewport_size.x / 2).cast(),
            (self.state.viewport_size.y - 10).cast(),
        );

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
                        ui.label(egui::RichText::new(format!("{pixel_legend:.3e}")));
                    });
            });
        });
    }

    pub(crate) fn fps_window(ctx: &egui::Context, ui_state: EsrUiState) {
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
            .fixed_pos([
                f32::conv_approx(pos.x) / scale,
                f32::conv_approx(pos.y) / scale,
            ])
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

    /// Atomically check if a load/save operation is active, and if not, set it to active.
    /// We use this to prevent multiple save/load operations from happening at once, which would
    /// make for a confusing UX.
    ///
    /// Returns true if the flag was clear and we set it.
    /// Returns false if the flag was already set.
    fn try_set_load_save_active(&self) -> bool {
        self.load_save_active
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
    }

    /// Determine the default directory to open the load/save dialog in,
    /// based on the last used directory (if we have one) or a provided default.
    /// As a final fallback, we use the current directory.
    ///
    /// `default` is a closure which returns Some(directory) if it can provide a
    /// reasonable default, or None if it can't.
    fn default_load_save_dir(&self, default: impl FnOnce() -> Option<PathBuf>) -> PathBuf {
        self.last_save_dir
            .as_ref()
            .filter(|dir| dir.is_dir())
            .cloned()
            .unwrap_or_else(|| {
                default()
                    .or_else(dirs::desktop_dir)
                    .unwrap_or_else(|| PathBuf::from("."))
            })
    }

    pub(crate) fn save_image_ui(&mut self, _ctx: &egui::Context) {
        self.show_save = false;
        if self.try_set_load_save_active() {
            let default_filename = format!(
                "brot3_{datetime}_{description}.png",
                datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
                description = self.state.display_string('_'),
            );

            let save_dialog = FileDialog::new()
                .add_filter("PNG image", &["png"])
                .set_title("Save image")
                .set_directory(self.default_load_save_dir(dirs::picture_dir))
                .set_file_name(&default_filename);

            let perturbation_points = self.perturbation.points.clone();
            let state = self.state.clone();
            let save_busy = Arc::clone(&self.save_busy);

            self.load_save_generic_workflow(
                || save_dialog.save_file(),
                move |filename| {
                    save_busy.store(true, std::sync::atomic::Ordering::Release);
                    scopeguard::defer! {
                        save_busy.store(false, std::sync::atomic::Ordering::Release);
                    }
                    save::do_save_image(
                        filename,
                        &state,
                        &perturbation_points,
                        RenderMode::default(),
                    )
                },
                "saving image",
            );
            // N.B. load_save_active is cleared by save_something's async task
        }
    }

    pub(crate) fn save_position_ui(&mut self, _ctx: &egui::Context) {
        self.show_save_position = false;
        if self.try_set_load_save_active() {
            let default_filename = format!(
                "brot3_{datetime}.json",
                datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
            );

            let save_dialog = FileDialog::new()
                .add_filter("JSON file", &["json"])
                .set_title("Save position")
                .set_directory(self.default_load_save_dir(dirs::document_dir))
                .set_file_name(&default_filename);

            let state = self.state.clone();
            self.load_save_generic_workflow(
                || save_dialog.save_file(),
                move |filename| state.save(filename).map_err(LoadSaveError::Lib),
                "saving state",
            );
        }
    }

    fn load_save_generic_workflow<DialogFn, ActionFn, R>(
        &self,
        run_dialog: DialogFn,
        do_action: ActionFn,
        action_verbing: &str,
    ) -> JoinHandle<Option<R>>
    where
        DialogFn: FnOnce() -> Option<PathBuf> + Send + 'static,
        ActionFn: FnOnce(&Path) -> Result<R, LoadSaveError> + Send + 'static,
        R: Send + 'static,
    {
        let load_save_active = Arc::clone(&self.load_save_active);
        let save_dir_channel = self.save_dir_channel.sender();
        let error_message_channel = self.error_message_channel.sender();
        let what = action_verbing.to_owned();
        tokio::task::spawn_blocking(move || {
            scopeguard::defer! {
                load_save_active.store(false, std::sync::atomic::Ordering::Release);
            }

            let Some(path) = run_dialog() else {
                log::info!("{what}: file dialog cancelled by user");
                return None;
            };
            match do_action(&path) {
                Ok(res) => {
                    if let Some(parent) = path.parent() {
                        save_dir_channel.send(parent.to_path_buf()).ok();
                    }
                    Some(res)
                }
                Err(err) => {
                    log::error!("{err}");
                    error_message_channel
                        .send(format!("Error {what}: {err}"))
                        .ok();
                    None
                }
            }
        })
    }

    pub(crate) fn open_ui(&mut self, _ctx: &egui::Context) {
        self.show_open = false;
        if self.try_set_load_save_active() {
            let open_dialog = FileDialog::new()
                .add_filter("JSON file or PNG image", &["json", "png"])
                .set_title("Open position or image")
                .set_directory(self.default_load_save_dir(|| Some(".".into())));

            self.loading_task = Some(self.load_save_generic_workflow(
                || open_dialog.pick_file(),
                |path| UiState::load_magic(path).map_err(LoadSaveError::Lib),
                "loading",
            ));
        }
    }

    pub(crate) fn error_modal(&mut self, ctx: &egui::Context) {
        if let Some(message) = &self.error_message {
            let message = message.clone();
            let _ = egui::Modal::new("error".into()).show(ctx, |ui| {
                ui.label(egui::RichText::new("Error").size(18.));
                ui.add_space(12.);
                ui.label(message.as_ref());
                ui.add_space(12.);
                if ui.button("OK").clicked() {
                    self.error_message = None;
                }
            });
        }
    }

    pub(crate) fn service_channels(&mut self) {
        use brot3_lib::ui::update_field_from_channel as uffc;
        uffc(&self.error_message_channel, &mut self.error_message);
        uffc(&self.save_dir_channel, &mut self.last_save_dir);
    }

    pub(crate) fn save_busy_window(ctx: &egui::Context) {
        egui::Window::new("Saving...")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(50., 25.))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Saving...").size(36.));
            });
    }
}
