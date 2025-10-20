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
    // Viewport position and movement
    viewport_translate: BigVec2,
    viewport_zoom: f64,
    movement: Movement,
    // Fractal detail
    algorithm: Algorithm,
    max_iter: u32,
    palette: Palette,
    exponent: Exponent,
    fractional_iters: bool,
    // User-facing options
    show_coords_window: bool,
    show_scale_bar: bool,
    show_fps: bool,
    vsync: bool,
    show_ui: bool,
    keyboard_help: bool,
    show_about: bool,
    show_license: bool,

    // UI operational data
    last_instant: Instant,
    mouse_position: DVec2,
    reiterate: bool,
    dragging: bool,
    ctrl_pressed: bool,
    resized: bool,
}

impl Controller {
    pub fn new(_options: &Options) -> Self {
        Self {
            size: UVec2::ZERO,
            // TODO figure out what precision is best
            viewport_translate: BigVec2::try_new(-1., 0.).unwrap().with_precision(PRECISION),
            viewport_zoom: 0.25,
            movement: Movement::default(),

            algorithm: Algorithm::default(),
            max_iter: 100,
            palette: Palette::default(),
            exponent: Exponent::default(),
            fractional_iters: true,

            show_coords_window: true,
            show_scale_bar: true,
            show_fps: false,
            vsync: true,
            show_ui: true,
            keyboard_help: false,
            show_about: false,
            show_license: false,

            last_instant: Instant::now(),
            mouse_position: DVec2::default(),
            reiterate: true,
            dragging: false,
            ctrl_pressed: false,
            resized: true,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Exponent {
    pub(crate) value: f32,
    pub(crate) value_i: u32,
    pub(crate) is_integer: bool,
    pub(crate) is_negative: bool,
}
impl Default for Exponent {
    fn default() -> Self {
        Self {
            value: 2.0,
            value_i: 2,
            is_integer: true,
            is_negative: false,
        }
    }
}
impl Exponent {
    fn variant(&self) -> NumericType {
        if self.is_integer {
            NumericType::Integer
        } else {
            NumericType::Float
        }
    }
    fn as_int(&self) -> i32 {
        self.value as i32 * if self.is_negative { -1 } else { 1 }
    }
    fn as_float(&self) -> f32 {
        self.value * if self.is_negative { -1. } else { 1. }
    }
    fn step(&self) -> f32 {
        if self.is_integer { 1. } else { 0.1 }
    }
}
impl From<Exponent> for PushExponent {
    fn from(exp: Exponent) -> Self {
        if exp.is_integer {
            PushExponent {
                typ: NumericType::Integer,
                int: exp.as_int(),
                float: 0.,
            }
        } else {
            PushExponent {
                typ: NumericType::Float,
                int: 0,
                float: exp.as_float(),
            }
        }
    }
}

#[derive(Default)]
struct Movement {
    translate: DVec2,
    zoom: f64,
    exponent: f32,
    gradient: f32,
    offset: f32,
    gamma: f32,
    saturation: f32,
    lightness: f32,
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
        self.reiterate = true;
        self.resized = true;
    }

    fn prepare_render(
        &mut self,
        _gfx_ctx: &GraphicsContext,
        _offset: Vec2,
    ) -> impl bytemuck::NoUninit {
        let reiterate = self.reiterate;
        self.reiterate = false;
        FragmentConstants {
            viewport_translate: self.viewport_translate.as_vec2(),
            viewport_zoom: self.viewport_zoom as f32,
            size: self.size.into(),
            algorithm: self.algorithm,
            max_iter: self.max_iter,
            needs_reiterate: reiterate.into(),
            exponent: self.exponent.into(),
            palette: self.palette,
            fractional_iters: self.fractional_iters.into(),
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
            contents: &[0; std::mem::size_of::<PointResult>()
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

    #[cfg(all(feature = "hot-reload-shader", not(wasm)))]
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

impl Controller {
    #[allow(dead_code)]
    pub(crate) fn viewport_complex_size(&self) -> DVec2 {
        // CAUTION: This must align with what shader is doing.
        self.size.as_dvec2() / (self.size.y as f64 * self.viewport_zoom)
    }
    pub(crate) fn pixel_complex_size(&self) -> f64 {
        // self.viewport_complex_size().y / self.size.y as f64
        1. / (self.viewport_zoom * self.size.y as f64)
    }
}
