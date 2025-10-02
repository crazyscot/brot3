use crate::Options;
use easy_shader_runner::{ControllerTrait, GraphicsContext, wgpu};
use engine3_common::*;
use glam::*;

pub(crate) struct Controller {
    /// viewport pixel size
    size: UVec2,
    reiterate: bool,
}

impl Controller {
    pub fn new(_options: &Options) -> Self {
        Self {
            size: UVec2::ZERO,
            reiterate: true,
        }
    }
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
}
