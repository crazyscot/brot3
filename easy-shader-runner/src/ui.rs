use std::sync::Arc;

use egui::{
    Context,
    epaint::{ClippedPrimitive, textures::TexturesDelta},
};
use egui_winit::{
    State,
    winit::{event::WindowEvent, window::Window},
};
use num_traits::AsPrimitive;

use crate::{GraphicsContext, controller::ControllerTrait, fps_counter::FpsCounter};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub escape_exits: bool,
    pub default_shader_key: Option<&'static str>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            escape_exits: true,
            default_shader_key: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UiState {
    fps: u32,
    #[cfg(not(target_arch = "wasm32"))]
    pub vsync: bool,

    /// Fullscreen is tricky!
    /// On macOS, the keypress Ctrl+Command+F (fullscreen) is handled by the OS.
    /// In other words, the fullscreen state may change for reasons we can't otherwise detect.
    ///
    /// easy-shader-runner sets `fullscreen_active` to reflect the actual fullscreen state.
    pub fullscreen_active: bool,
    /// Controller sets this when it wants to change the state. easy-shader-runner will clear it
    /// once actioned.
    pub fullscreen_requested: Option<bool>,
    pub escape_exits: bool,

    pub last_elapsed: Option<std::time::Duration>,
}

impl UiState {
    #[must_use]
    pub fn new(options: Options) -> Self {
        Self {
            fps: 0,
            #[cfg(not(target_arch = "wasm32"))]
            vsync: true,
            fullscreen_active: false,
            fullscreen_requested: None,
            escape_exits: options.escape_exits,
            last_elapsed: None,
        }
    }

    #[must_use]
    pub fn fps(&self) -> &u32 {
        &self.fps
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new(Options::default())
    }
}

pub(crate) struct Ui {
    egui_winit_state: State,
    fps_counter: FpsCounter,
}

impl Ui {
    pub(crate) fn new(window: &Arc<Window>) -> Self {
        let context = Context::default();
        context.options_mut(|w| w.zoom_with_keyboard = false);
        let viewport_id = context.viewport_id();
        let egui_winit_state = State::new(
            context,
            viewport_id,
            &window,
            Some(window.scale_factor().as_()),
            None,
            None,
        );

        Self {
            egui_winit_state,
            fps_counter: FpsCounter::new(),
        }
    }

    pub(crate) fn consumes_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_winit_state
            .on_window_event(window, event)
            .consumed
    }

    pub(crate) fn prepare<C: ControllerTrait>(
        &mut self,
        window: &Window,
        ui_state: &mut UiState,
        controller: &mut C,
        graphics_context: &GraphicsContext,
    ) -> (Vec<ClippedPrimitive>, TexturesDelta, egui::Rect, f32) {
        ui_state.fps = self.fps_counter.tick();
        let raw_input = self.egui_winit_state.take_egui_input(window);
        let mut available_rect = egui::Rect::NAN;
        let full_output = self.egui_winit_state.egui_ctx().run_ui(raw_input, |ctx| {
            Self::ui(ctx, ui_state, controller, graphics_context);
            available_rect = ctx.content_rect();
        });
        self.egui_winit_state
            .handle_platform_output(window, full_output.platform_output);
        let clipped_primitives = self
            .egui_winit_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        (
            clipped_primitives,
            full_output.textures_delta,
            available_rect,
            self.egui_winit_state.egui_ctx().pixels_per_point(),
        )
    }

    fn ui<C: ControllerTrait>(
        ctx: &Context,
        ui_state: &mut UiState,
        controller: &mut C,
        graphics_context: &GraphicsContext,
    ) {
        controller.ui(ctx, ui_state, graphics_context);
    }
}
