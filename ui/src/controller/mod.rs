use crate::cli::Args;

use easy_shader_runner::{egui, wgpu, winit, ControllerTrait, GraphicsContext, UiState};
use glam::{dvec2, uvec2, DVec2, UVec2, Vec2};
use shader_common::{
    data::PointResult, enums::Algorithm, flag_if, Flags, FragmentConstants, NumericType, Palette,
    PushExponent,
};
use util::BigVec2;
use web_time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;

mod about;
mod controls;
mod coords;
mod keyboard;
mod menu;
mod small_windows;
mod ui;

const PRECISION: usize = 128;
const MIN_ZOOM: f64 = 0.05;
const MAX_ZOOM: f64 = 13000.; // TODO: implement perturbed mbrot

pub(crate) struct Controller {
    /// viewport size in pixels
    size: UVec2,
    cache_size: UVec2,
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
    show_controls: bool,
    keyboard_help: bool,
    show_about: bool,
    show_license: bool,

    // UI operational data
    last_instant: Instant,
    mouse_position: DVec2,
    reiterate: bool,
    always_reiterate: bool,
    dragging: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    super_pressed: bool,
    resized: bool,
    fullscreen_checkbox: bool,
    fullscreen_requested: Option<bool>,
    context_menu: Option<DVec2>,
    inspector: Inspector,
    render_pass: u32,
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
    /// This function exists to provide a _build-time_ dependency on the `png` feature of the `image` crate.
    /// It is not called.
    #[allow(dead_code)]
    fn dummy_dependency() {
        let slice = &[0u8; 1];
        let rdr = std::io::BufReader::new(std::io::Cursor::new(slice));
        let _ = image::codecs::png::PngDecoder::new(rdr);
    }

    pub fn new(options: &Args) -> Self {
        Self {
            size: UVec2::ZERO,
            cache_size: options.cache_size.unwrap_or_default().into(),
            // TODO figure out what precision is best
            viewport_translate: BigVec2::try_new(-1., 0.).unwrap().with_precision(PRECISION),
            viewport_zoom: FragmentConstants::DEFAULT_ZOOM.into(),
            movement: Movement::default(),

            algorithm: options.fractal,
            max_iter: FragmentConstants::DEFAULT_MAX_ITER,
            palette: Palette::default().with_colourer(options.colourer), // TODO with render style too
            exponent: Exponent::default(),

            show_coords_window: true,
            show_scale_bar: true,
            show_fps: false,
            vsync: true,
            show_controls: !options.no_ui,
            keyboard_help: false,
            show_about: false,
            show_license: false,

            last_instant: Instant::now(),
            mouse_position: DVec2::default(),
            reiterate: true,
            always_reiterate: false,
            dragging: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            super_pressed: false,
            resized: true,
            fullscreen_checkbox: options.fullscreen,
            fullscreen_requested: Some(options.fullscreen),
            context_menu: None,
            inspector: Inspector::default(),
            render_pass: 0,
        }
    }

    fn fragment_constants(&self, reiterate: bool) -> FragmentConstants {
        let flags = flag_if(reiterate || self.always_reiterate, Flags::NEEDS_REITERATE)
            | flag_if(self.inspector.active, Flags::INSPECTOR_ACTIVE);
        FragmentConstants {
            flags,
            viewport_translate: self.viewport_translate.as_vec2(),
            viewport_zoom: self.viewport_zoom as f32,
            size: self.size.into(),
            buffer_size: self.cache_size.into(),
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
        self.inspector.stale |= reiterate;
        self.reiterate = false;
        self.fragment_constants(reiterate)
    }

    fn describe_wgpu_features_and_limits(
        &self,
        _supported_features: wgpu::Features,
        supported_limits: wgpu::Limits,
    ) -> (wgpu::Features, wgpu::Limits) {
        let max_storage_buffer_binding_size =
            core::mem::size_of::<PointResult>() as u32 * self.cache_size.element_product();
        let max_buffer_size = max_storage_buffer_binding_size.into();
        assert!(max_buffer_size < supported_limits.max_buffer_size);
        assert!(max_storage_buffer_binding_size < supported_limits.max_storage_buffer_binding_size);
        (
            wgpu::Features::default(),
            wgpu::Limits {
                max_storage_buffer_binding_size,
                max_buffer_size,
                ..Default::default()
            },
        )
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
        let cache_size = self.cache_size;
        log::info!("Using cache size {cache_size}");
        assert!(
            cache_size != UVec2::ZERO,
            "logic error: cache_size was not set up by the time we needed it"
        );
        let initial_contents =
            vec![0; std::mem::size_of::<PointResult>() * cache_size.element_product() as usize];
        let render_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render_data_buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: &initial_contents,
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
            self.viewport_translate += delta * self.modifier_key_factor() / self.viewport_zoom;
            self.reiterate = true;
        }
    }
    fn mouse_scroll(&mut self, delta: DVec2) {
        if delta.y == 0. {
            return;
        }

        let motion = delta.y * 0.1 * self.modifier_key_factor();
        let position = self.mouse_position;
        let size = self.size.as_dvec2();
        let prev_zoom = self.viewport_zoom;
        let zoom = &mut self.viewport_zoom;
        let mouse_pos0 = BigVec2::try_from(position - size / 2.).unwrap() / *zoom / size.y;
        *zoom = (prev_zoom * (1.0 + motion)).clamp(MIN_ZOOM, MAX_ZOOM);
        let mouse_pos1 = BigVec2::try_from(position - size / 2.).unwrap() / *zoom / size.y;
        self.viewport_translate += mouse_pos0 - mouse_pos1;
        self.reiterate = true;
    }

    /// Early app callback, before the window is created.
    ///
    /// Find the largest fullscreen size available to us
    fn app_resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut biggest = PhysicalSize::new(800, 600); // fallback in case detection fails

        for mon in event_loop.available_monitors() {
            for mode in mon.video_modes() {
                let size = mode.size();
                if size.width >= biggest.width && size.height >= biggest.height {
                    biggest = size;
                }
            }
        }
        log::info!(
            "Largest available screen is {} x {}",
            biggest.width,
            biggest.height
        );
        if self.cache_size != UVec2::ZERO {
            log::info!("Cache size override active");
        } else {
            self.cache_size = uvec2(biggest.width, biggest.height);
        }
    }
}

impl Controller {
    pub(crate) fn pixel_complex_size(&self) -> f64 {
        // This must be the same calculation that the shader uses.
        FragmentConstants::pixel_spacing_f64(self.size.y, self.viewport_zoom)
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
