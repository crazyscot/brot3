use crate::Options;
use easy_shader_runner::{ControllerTrait, GraphicsContext, UiState, egui, wgpu, winit};
use glam::*;
use shader_common::*;
use shader_util::big_vec2::BigVec2;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};

mod keyboard;
mod ui;

const PRECISION: usize = 128;
const MAX_ZOOM: f64 = 13000.; // TODO: implement perturbed mbrot

pub(crate) struct Controller {
    /// viewport pixel size
    size: UVec2,
    reiterate: bool,
    viewport_translate: BigVec2,
    viewport_zoom: f64,
    movement: Movement,
    last_instant: Instant,
    mouse_position: DVec2,
    show_coords_window: bool,
    max_iter: u32,
    show_fps: bool,
    dragging: bool,
}

impl Controller {
    pub fn new(_options: &Options) -> Self {
        Self {
            size: UVec2::ZERO,
            reiterate: true,
            // TODO figure out what precision is best
            viewport_translate: BigVec2::try_new(0., 0.).unwrap().with_precision(PRECISION),
            viewport_zoom: 0.25,
            movement: Movement::default(),
            last_instant: Instant::now(),
            mouse_position: DVec2::default(),
            show_coords_window: true,
            max_iter: 100,
            show_fps: true, // TODO will become an option
            dragging: false,
        }
    }
}

#[derive(Default)]
struct Movement {
    translate: DVec2,
    zoom: f64,
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
        self.reiterate = true;
    }

    fn prepare_render(
        &mut self,
        _gfx_ctx: &GraphicsContext,
        _offset: Vec2,
    ) -> impl bytemuck::NoUninit {
        let reiterate = self.reiterate;
        self.reiterate = false;
        FragmentConstants {
            size: self.size.into(),
            needs_reiterate: reiterate.into(),
            viewport_translate: self.viewport_translate.as_vec2(),
            viewport_zoom: self.viewport_zoom as f32,
            max_iter: self.max_iter,
        }
    }

    fn describe_bind_groups(
        &mut self,
        gfx_ctx: &GraphicsContext,
    ) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
        let device = &gfx_ctx.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind_group_layout"),
        });

        use wgpu::util::DeviceExt;
        let render_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render_data_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            contents: &[0; std::mem::size_of::<RenderData>()
                * GRID_SIZE.x as usize
                * GRID_SIZE.y as usize],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: render_data_buffer.as_entire_binding(),
            }],
            label: Some("fractal_bind_group"),
        });
        (vec![layout], vec![bind_group])
    }

    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    fn new_shader_module(&mut self) {
        self.reiterate = true;
    }

    fn ui(&mut self, ctx: &egui::Context, ui_state: &mut UiState, gfx_ctx: &GraphicsContext) {
        self.ui_impl(ctx, ui_state, gfx_ctx);
    }

    fn keyboard_input(&mut self, key: winit::event::KeyEvent) {
        self.keyboard_input_impl(key);
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.dragging = state == ElementState::Pressed;
        }
    }
    fn mouse_move(&mut self, position: DVec2) {
        let prev_position = self.mouse_position;
        self.mouse_position = position;
        if self.dragging {
            let delta =
                BigVec2::try_from((prev_position - self.mouse_position) / self.size.y as f64)
                    .unwrap()
                    .with_precision(PRECISION);
            self.viewport_translate += delta / self.viewport_zoom;
            self.reiterate = true;
        }
    }
    fn mouse_scroll(&mut self, delta: DVec2) {
        if delta.y == 0. {
            return;
        }
        let motion = delta.y * 0.1;
        let position = self.mouse_position;
        let size = self.size.as_dvec2();
        let prev_zoom = self.viewport_zoom;
        let zoom = &mut self.viewport_zoom;
        let mouse_pos0 = BigVec2::try_from(position - size / 2.).unwrap() / *zoom / size.y;
        *zoom = (prev_zoom * (1.0 + motion)).clamp(0.05, MAX_ZOOM);
        let mouse_pos1 = BigVec2::try_from(position - size / 2.).unwrap() / *zoom / size.y;
        self.viewport_translate += mouse_pos0 - mouse_pos1;
        self.reiterate = true;
    }
}
