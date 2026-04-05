use crate::controller::ControllerTrait;
use egui_winit::winit::{dpi::PhysicalSize, window::Window};
use std::sync::Arc;

pub struct GraphicsContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl GraphicsContext {
    pub async fn new<C: ControllerTrait>(
        window: Arc<Window>,
        initial_size: PhysicalSize<u32>,
        controller: &C,
    ) -> GraphicsContext {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                flags: wgpu::InstanceFlags::default(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
                backend_options: wgpu::BackendOptions::default(),
            }
            .with_env(),
        );

        #[cfg(target_arch = "wasm32")]
        let canvas = {
            use egui_winit::winit::platform::web::*;
            window.canvas().unwrap()
        };
        let initial_surface = instance.create_surface(window);
        #[cfg(target_arch = "wasm32")]
        if initial_surface.is_err() {
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    doc.body().and_then(|body| {
                        let element = doc.create_element("span").unwrap();
                        element.set_inner_html("Your browser does not support WebGPU");
                        element.set_id("incompatible_no_webgpu");
                        body.replace_child(&element.into(), &canvas.into()).ok()
                    })
                })
                .expect("couldn't append message to document body");
        }
        let initial_surface = initial_surface.expect("Failed to create surface from window");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&initial_surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (features, limits) =
            controller.describe_wgpu_features_and_limits(adapter.features(), adapter.limits());
        let (features, limits) = if cfg!(feature = "emulate_constants") {
            (features, limits)
        } else {
            (
                features | wgpu::Features::PUSH_CONSTANTS,
                wgpu::Limits {
                    max_push_constant_size: limits.max_push_constant_size.max(128),
                    ..limits
                },
            )
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: features,
                required_limits: limits,
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        fn auto_configure_surface<'a>(
            adapter: &wgpu::Adapter,
            device: &wgpu::Device,
            surface: wgpu::Surface<'a>,
            size: PhysicalSize<u32>,
        ) -> (wgpu::Surface<'a>, wgpu::SurfaceConfiguration) {
            let capabilities = surface.get_capabilities(adapter);
            let mut surface_config = surface
                .get_default_config(adapter, size.width, size.height)
                .unwrap_or_else(|| {
                    panic!(
                        "Missing formats/present modes in surface capabilities: {capabilities:#?}"
                    )
                });
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;
            surface_config.format =
                egui_wgpu::preferred_framebuffer_format(&capabilities.formats).unwrap();
            surface.configure(device, &surface_config);
            (surface, surface_config)
        }

        let (surface, config) =
            auto_configure_surface(&adapter, &device, initial_surface, initial_size);

        GraphicsContext {
            surface,
            device,
            queue,
            config,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_vsync(&mut self, enable: bool) {
        let present_mode = if enable {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        if self.config.present_mode != present_mode {
            self.config.present_mode = present_mode;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
