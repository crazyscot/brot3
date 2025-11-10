use crate::cli::Args;

use easy_shader_runner::{egui, wgpu, winit, ControllerTrait, GraphicsContext, UiState};
use glam::{dvec2, DVec2, UVec2, Vec2};
use shader_common::{
    data::{PointResult, PointResultA, PointResultB},
    enums::Algorithm,
    Flags, FragmentConstants, NumericType, Palette, PushExponent, GRID_SIZE,
};
use util::BigVec2;
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
    fullscreen_requested: bool,
    context_menu: Option<DVec2>,
    inspector: Inspector,
}

#[derive(Default)]
struct Inspector {
    active: bool,
    dragging: bool,
    position: BigVec2,
    stale: bool, // N.B. Controller.reiterate implies the inspector data is stale. The converse is not true.
    data: PointResult,
}

impl Controller {
    pub fn new(options: &Args) -> Self {
        Self {
            size: UVec2::ZERO,
            // TODO figure out what precision is best
            viewport_translate: BigVec2::try_new(-1., 0.).unwrap().with_precision(PRECISION),
            viewport_zoom: 0.25,
            movement: Movement::default(),

            algorithm: options.fractal,
            max_iter: 250,
            palette: Palette::default().with_colourer(options.colourer), // TODO with render style too
            exponent: Exponent::default(),

            show_coords_window: true,
            show_scale_bar: true,
            show_fps: false,
            vsync: true,
            show_ui: !options.no_ui,
            keyboard_help: false,
            show_about: false,
            show_license: false,

            last_instant: Instant::now(),
            mouse_position: DVec2::default(),
            reiterate: true,
            dragging: false,
            ctrl_pressed: false,
            resized: true,
            fullscreen_requested: options.fullscreen,
            context_menu: None,
            inspector: Inspector::default(),
        }
    }
    fn fragment_constants(&self, reiterate: bool) -> FragmentConstants {
        let flags = if reiterate {
            Flags::NEEDS_REITERATE
        } else {
            Flags::empty()
        } | if self.inspector.active {
            Flags::INSPECTOR_ACTIVE
        } else {
            Flags::empty()
        };
        FragmentConstants {
            flags,
            viewport_translate: self.viewport_translate.as_vec2(),
            viewport_zoom: self.viewport_zoom as f32,
            size: self.size.into(),
            algorithm: self.algorithm,
            max_iter: self.max_iter,
            exponent: self.exponent.into(),
            palette: self.palette,
            inspector_point_pixel_address: self
                .complex_point_to_pixel(&self.inspector.position)
                .as_vec2(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Exponent {
    pub(crate) int: u32,
    pub(crate) real: f32,
    pub(crate) imag: f32,
    pub(crate) typ: NumericType,
    pub(crate) real_is_negative: bool,
    pub(crate) imag_is_negative: bool,
}
impl Default for Exponent {
    fn default() -> Self {
        Self {
            int: 2,
            real: 2.0,
            imag: 0.0,
            typ: NumericType::Integer,
            real_is_negative: false,
            imag_is_negative: false,
        }
    }
}
impl Exponent {
    fn variant(&self) -> NumericType {
        self.typ
    }
    fn step(&self) -> f32 {
        if self.typ == NumericType::Integer {
            1.
        } else {
            0.1
        }
    }
    fn is_integer(&self) -> bool {
        self.typ == NumericType::Integer
    }
}
impl From<Exponent> for PushExponent {
    // TODO: Can we merge Exponent and PushExponent?
    fn from(exp: Exponent) -> Self {
        match exp.typ {
            NumericType::Integer => PushExponent {
                typ: NumericType::Integer,
                int: exp.real as i32 * if exp.real_is_negative { -1 } else { 1 },
                ..Default::default()
            },
            NumericType::Float => PushExponent {
                typ: NumericType::Float,
                real: exp.real * if exp.real_is_negative { -1. } else { 1. },
                ..Default::default()
            },
            NumericType::Complex => PushExponent {
                typ: NumericType::Complex,
                real: exp.real * if exp.real_is_negative { -1. } else { 1. },
                imag: exp.imag * if exp.imag_is_negative { -1. } else { 1. },
                ..Default::default()
            },
            _ => todo!(),
        }
    }
}

#[derive(Default)]
struct Movement {
    translate: DVec2,
    zoom: f64,
    exponent: f32,
    exponent_im: f32,
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
        self.inspector.stale = reiterate;
        self.reiterate = false;
        self.fragment_constants(reiterate)
    }

    fn describe_bind_groups(
        &mut self,
        gfx_ctx: &GraphicsContext,
    ) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
        let device = &gfx_ctx.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

        use wgpu::util::DeviceExt;
        let render_data_buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render_data_buffer_a"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: &[0; std::mem::size_of::<PointResultA>()
                * GRID_SIZE.x as usize
                * GRID_SIZE.y as usize],
        });
        let render_data_buffer_b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render_data_buffer_b"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: &[0; std::mem::size_of::<PointResultB>()
                * GRID_SIZE.x as usize
                * GRID_SIZE.y as usize],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_data_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: render_data_buffer_b.as_entire_binding(),
                },
            ],
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
        let pressed = state == ElementState::Pressed;
        match button {
            MouseButton::Left => {
                self.dragging = pressed;
                self.inspector.dragging = pressed && self.mouse_on_marker();
            }
            MouseButton::Right => {
                if state == ElementState::Pressed {
                    // hack: offset the menu from the clicked point, so it doesn't immediately disappear
                    self.context_menu = Some(self.mouse_position - DVec2::splat(5.0));
                }
            }
            _ => (),
        }
    }
    fn mouse_move(&mut self, position: DVec2) {
        let prev_position = self.mouse_position;
        self.mouse_position = position;
        if self.inspector.dragging {
            self.inspector.position += self.pixel_address_to_complex(self.mouse_position)
                - self.pixel_address_to_complex(prev_position);
            self.inspector.stale = true;
        } else if self.dragging {
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

    fn pixel_address_to_complex(&self, p: DVec2) -> BigVec2 {
        let size = self.size.as_dvec2();
        self.viewport_translate.clone()
            + BigVec2::try_from(
                (p - 0.5 * size) * dvec2(size.x / size.y, 1.0) / self.viewport_zoom / size,
            )
            .unwrap()
    }

    fn complex_point_to_pixel(&self, p: &BigVec2) -> DVec2 {
        let size = self.size.as_dvec2();
        (p.clone() - self.viewport_translate.clone()).as_dvec2() / dvec2(size.x / size.y, 1.0)
            * self.viewport_zoom
            * size
            + 0.5 * size
    }
}
