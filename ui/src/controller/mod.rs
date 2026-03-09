//! Controller that plugs into the [`easy_shader_runner`] framework.
//! It manages the application state and UI.
//!
//! <div class="warning">
//! This crate has a hidden dependency on the `png` feature of the `image` crate.
//! </div>

use std::sync::{Arc, Mutex};

use base::{BigVec2, PixelSpacing as _, PushExponent, enums::Algorithm, ui::ViewportZoom};
use easy_shader_runner::{ControllerTrait, GraphicsContext, UiState, egui, wgpu, winit};
use glam::{DVec2, UVec2, Vec2, dvec2, uvec2};
use shader::{
    data::PointResult,
    push_constants::{Flags, FragmentConstants, Palette},
};
use web_time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton},
    event_loop::ActiveEventLoop,
};

use crate::cli::Args;

mod about;
mod controls;
mod coords;
mod keyboard;
mod menu;
mod small_windows;
mod ui;

// dashu uses whatever actual digit size it considers necessary, up to this limit.
// Larger limits reduce performance in deep zooms, but may improve accuracy.
const BIGNUM_PRECISION_LIMIT: usize = 192;

const MIN_ZOOM: f64 = 0.05;
const MAX_ZOOM_STANDARD: f64 = 1.0e4; // reported on UI as 40000

// Around this point, f32 maths breaks down: we can no longer accurately represent pixel sizes.
const MAX_ZOOM_PERTURBATIONS_F32: f64 = 2.5e34; // reported on UI as 1e35

// N.B. This affects the perturbation buffer size. But it's only 2 * sizeof(f32) per point.
const MAX_MAX_ITERATIONS: u32 = 100_000;

#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Controller {
    /// viewport size in pixels
    size: UVec2,
    cache_size: UVec2,
    // Viewport position and movement
    viewport_translate: BigVec2,
    viewport_zoom: ViewportZoom,
    movement: Movement,
    // Fractal detail
    algorithm: Algorithm,
    max_iter: u32,
    palette: Palette,
    exponent: PushExponent,
    perturbation: PerturbationReference,
    iteration_cull: bool,

    // User-facing options
    show_coords_window: bool,
    show_scale_bar: bool,
    show_fps: bool,
    vsync: bool,
    show_controls: bool,
    keyboard_help: bool,
    show_about: bool,
    show_license: bool,
    show_save: bool,

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
    perturbation_mode: bool,
    force_perturb: bool,
    fullscreen_checkbox: bool,
    fullscreen_requested: Option<bool>,
    context_menu: Option<DVec2>,
    inspector: Inspector,
    render_pass: u32,
    save_active: Arc<Mutex<bool>>,
    last_save_dir: Arc<Mutex<Option<std::path::PathBuf>>>,
    error_message: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
struct Inspector {
    active: bool,
    dragging: bool,
    position: BigVec2,
    stale: bool, /* N.B. Controller.reiterate implies the inspector data is stale. The converse
                  * is not true. */
    data: PointResult,
}

impl Controller {
    /// The size of the complex plane that you see from [`FragmentConstants::DEFAULT_ZOOM`] with the
    /// default window size
    pub(crate) const DEFAULT_FRACTAL_PLANE_SIZE: f64 = 4.0;
    /// The window size that defines a zoom factor of 1.0.
    ///
    /// This happens to be what we get by default from winit on Linux.
    pub(crate) const NOMINAL_WINDOW_SIZE: UVec2 = uvec2(800, 600);

    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn new(options: &Args) -> Self {
        Self {
            size: UVec2::ZERO,
            cache_size: options.cache_size.unwrap_or_default().into(),
            // TODO figure out what precision is best; do we need to make it dynamic?
            viewport_translate: BigVec2::try_new(-1., 0.)
                .unwrap()
                .with_precision(BIGNUM_PRECISION_LIMIT),
            viewport_zoom: FragmentConstants::DEFAULT_ZOOM.into(),
            movement: Movement::default(),

            algorithm: options.fractal,
            max_iter: FragmentConstants::DEFAULT_MAX_ITER,
            palette: Palette::default().with_colourer(options.colourer), /* TODO with render
                                                                          * style too */
            exponent: PushExponent::default(),
            perturbation: PerturbationReference::default(),
            iteration_cull: false,

            show_coords_window: true,
            show_scale_bar: true,
            show_fps: false,
            vsync: true,
            show_controls: !options.no_ui,
            keyboard_help: false,
            show_about: false,
            show_license: false,
            show_save: false,

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
            perturbation_mode: false,
            force_perturb: false,
            fullscreen_checkbox: options.fullscreen,
            fullscreen_requested: Some(options.fullscreen),
            context_menu: None,
            inspector: Inspector::default(),
            render_pass: 0,
            save_active: Arc::new(Mutex::new(false)),
            last_save_dir: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn fragment_constants(&self, reiterate: bool) -> FragmentConstants {
        let flags = Flags::flag_if(reiterate || self.always_reiterate, Flags::NEEDS_REITERATE)
            | Flags::flag_if(self.inspector.active, Flags::INSPECTOR_ACTIVE)
            | Flags::flag_if(self.perturbation_mode, Flags::PERTURBATION_MODE)
            | Flags::flag_if(self.iteration_cull, Flags::ITERATION_CULL);
        FragmentConstants {
            flags,
            viewport_translate: self.viewport_translate.as_vec2(),
            viewport_zoom: self.viewport_zoom.into(),
            size: self.size.into(),
            buffer_size: self.cache_size.into(),
            algorithm: self.algorithm,
            max_iter: self.max_iter,
            exponent: self.exponent,
            palette: self.palette,
            inspector_point_pixel_address: self
                .complex_point_to_pixel(&self.inspector.position)
                .as_vec2(),
            n_reference_points: self.perturbation.points.len() as u32,
        }
    }

    /// Maximum zoom for the current settings
    fn zoom_max(&self) -> f64 {
        if self.perturbation_mode || self.force_perturb {
            MAX_ZOOM_PERTURBATIONS_F32
        } else {
            MAX_ZOOM_STANDARD
        }
    }

    fn apply_zoom_limits(&self, zoom: f64) -> f64 {
        zoom.clamp(MIN_ZOOM, self.zoom_max())
    }

    /// Apply a new zoom factor, subject to the limits, perturbation state, and possibility to
    /// auto-update the perturbation state.
    pub(crate) fn update_zoom_factor(&mut self, new_zoom: f64) {
        // Auto-update perturbation, if appropriate
        if !self.force_perturb {
            let zooming_in = new_zoom > self.viewport_zoom.0;
            if !self.perturbation_mode
                && zooming_in
                && new_zoom > MAX_ZOOM_STANDARD
                && self.perturb_implemented()
            {
                self.perturbation_mode = true;
            } else if self.perturbation_mode && !zooming_in && new_zoom < MAX_ZOOM_STANDARD {
                self.perturbation_mode = false;
            }
        }
        // Apply the limits
        self.viewport_zoom.0 = self.apply_zoom_limits(new_zoom);
    }
}

struct Movement {
    translate: DVec2,
    zoom2: f64, /* Specialised scale factor. 1.0 => do nothing; >1.0 zoom in by that factor; <
                 * -1.0 zoom out by negated factor; (-1.0..1.0) invalid. */
    exponent: f32,
    exponent_im: f32,
    gradient: f32,
    offset: f32,
    gamma: f32,
    saturation: f32,
    lightness: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            zoom2: 1.0,
            translate: DVec2::ZERO,
            exponent: Default::default(),
            exponent_im: Default::default(),
            gradient: Default::default(),
            offset: Default::default(),
            gamma: Default::default(),
            saturation: Default::default(),
            lightness: Default::default(),
        }
    }
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

    #[allow(clippy::cast_possible_truncation)]
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
        use wgpu::util::DeviceExt;

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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

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

        let perturbation_points_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("perturbation_points_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<Vec2>() as u64 * u64::from(MAX_MAX_ITERATIONS + 1),
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: perturbation_points_buffer.as_entire_binding(),
                },
            ],
            label: Some("fractal_bind_group"),
        });
        self.perturbation.buffer = Some(perturbation_points_buffer);
        self.perturbation
            .points
            .reserve(MAX_MAX_ITERATIONS as usize);
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
        self.keyboard_input_impl(&key);
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
                    // hack: offset the menu from the clicked point, so it doesn't immediately
                    // disappear
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
                - &self.pixel_address_to_complex(prev_position);
            self.inspector.stale = true;
        } else if self.dragging {
            let delta =
                BigVec2::try_from((prev_position - self.mouse_position) / f64::from(self.size.y))
                    .unwrap()
                    .with_precision(BIGNUM_PRECISION_LIMIT);
            self.viewport_translate += delta * self.modifier_key_factor() / self.viewport_zoom.0;
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
        let prev_zoom = self.viewport_zoom.0;
        let mouse_pos0 = BigVec2::try_from(position - size / 2.).unwrap() / prev_zoom / size.y;
        self.update_zoom_factor(prev_zoom * (1.0 + motion));
        let new_zoom = self.viewport_zoom.0;
        let mouse_pos1 = BigVec2::try_from(position - size / 2.).unwrap() / new_zoom / size.y;
        self.viewport_translate += &(mouse_pos0 - &mouse_pos1);
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
        if self.cache_size == UVec2::ZERO {
            self.cache_size = uvec2(biggest.width, biggest.height);
        } else {
            log::info!("Cache size override active");
        }
    }
}

impl Controller {
    pub(crate) fn pixel_complex_size(&self) -> f64 {
        // This must be the same calculation that the shader uses.
        self.viewport_zoom.0.pixel_spacing(self.size.y)
    }

    #[allow(clippy::missing_panics_doc)]
    fn pixel_address_to_complex(&self, p: DVec2) -> BigVec2 {
        let size = self.size.as_dvec2();
        BigVec2::try_from(
            (p - 0.5 * size) * dvec2(size.x / size.y, 1.0) / self.viewport_zoom.0 / size,
        )
        .unwrap()
            + &self.viewport_translate
    }

    fn complex_point_to_pixel(&self, p: &BigVec2) -> DVec2 {
        let size = self.size.as_dvec2();
        (p.clone() - &self.viewport_translate).as_dvec2() / dvec2(size.x / size.y, 1.0)
            * self.viewport_zoom.0
            * size
            + 0.5 * size
    }
}

#[derive(Default)]
struct PerturbationReference {
    buffer: Option<wgpu::Buffer>,
    points: Vec<Vec2>,
}
