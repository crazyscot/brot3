use crate::{
    context::GraphicsContext,
    controller::ControllerTrait,
    ui::{Ui, UiState},
};
use egui_winit::winit::window::Window;

#[cfg(feature = "emulate_constants")]
struct EmulateConstantsBuffer {
    render: wgpu::Buffer,
    #[cfg(feature = "compute")]
    compute: wgpu::Buffer,
}

struct Pipelines {
    render: wgpu::RenderPipeline,
    #[cfg(feature = "compute")]
    compute: wgpu::ComputePipeline,
}

struct PipelineLayouts {
    render: wgpu::PipelineLayout,
    #[cfg(feature = "compute")]
    compute: wgpu::PipelineLayout,
}

pub struct RenderPass {
    pipelines: Pipelines,
    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    pipeline_layouts: PipelineLayouts,
    ui_renderer: egui_wgpu::Renderer,
    bind_groups: Vec<wgpu::BindGroup>,
    shader_viewport: egui::Rect,
    #[cfg(feature = "emulate_constants")]
    emulate_constants_buffer: EmulateConstantsBuffer,
    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
}

impl RenderPass {
    pub fn new<C: ControllerTrait>(
        ctx: &GraphicsContext,
        shader_bytes: &[u8],
        controller: &mut C,
    ) -> Self {
        let (layouts, bind_groups) = controller.describe_bind_groups(ctx);
        let bind_group_layouts = layouts.iter();

        #[cfg(feature = "emulate_constants")]
        let (emulate_constants_layout, emulate_constants_bind_group, emulate_constants_buffer) =
            create_emulate_constants_bind_groups(&ctx.device);
        #[cfg(feature = "emulate_constants")]
        let bind_group_layouts = bind_group_layouts.chain([&emulate_constants_layout]);
        #[cfg(feature = "emulate_constants")]
        let bind_groups = bind_groups
            .into_iter()
            .chain([emulate_constants_bind_group])
            .collect::<Vec<_>>();

        let vertex_buffer_layouts = controller.describe_vertex_buffer_layouts(ctx);
        let pipeline_layouts =
            create_pipeline_layouts(ctx, &bind_group_layouts.collect::<Vec<_>>());
        let pipelines = create_pipelines(
            &ctx.device,
            &pipeline_layouts,
            ctx.config.format,
            &vertex_buffer_layouts,
            shader_bytes,
        );

        let ui_renderer = egui_wgpu::Renderer::new(
            &ctx.device,
            ctx.config.format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );

        Self {
            pipelines,
            #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
            pipeline_layouts,
            ui_renderer,
            bind_groups,
            shader_viewport: egui::Rect::NAN,
            #[cfg(feature = "emulate_constants")]
            emulate_constants_buffer,
            #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
            vertex_buffer_layouts,
        }
    }

    #[cfg(feature = "compute")]
    pub fn compute(
        &self,
        ctx: &GraphicsContext,
        dimensions: glam::UVec3,
        threads: glam::UVec3,
        push_constants: &[u8],
    ) {
        let workspace = (dimensions.as_vec3() / threads.as_vec3()).ceil().as_uvec3();
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            cpass.set_pipeline(&self.pipelines.compute);
            {
                #[cfg(not(feature = "emulate_constants"))]
                cpass.set_push_constants(0, push_constants);
                #[cfg(feature = "emulate_constants")]
                ctx.queue
                    .write_buffer(&self.emulate_constants_buffer.compute, 0, push_constants);
            }
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                cpass.set_bind_group(i as u32, bind_group, &[]);
            }
            cpass.dispatch_workgroups(workspace.x, workspace.y, workspace.z);
        }
        ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn render<C: ControllerTrait>(
        &mut self,
        ctx: &GraphicsContext,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut C,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = match ctx.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(err) => {
                eprintln!("get_current_texture error: {err:?}");
                return match err {
                    wgpu::SurfaceError::Lost => {
                        ctx.surface.configure(&ctx.device, &ctx.config);
                        Ok(())
                    }
                    _ => Err(err),
                };
            }
        };
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_ui(ctx, &output_view, window, ui, ui_state, controller);

        output.present();

        Ok(())
    }

    fn render_shader<C: ControllerTrait>(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &wgpu::TextureView,
        controller: &mut C,
        available_rect: egui::Rect,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shader Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shader Render Pass"),
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let size = glam::vec2(available_rect.width(), available_rect.height()).floor();
            if self.shader_viewport != available_rect {
                self.shader_viewport = available_rect;
                controller.resize(size.as_uvec2());
            }
            let offset = self.shader_offset();
            rpass.set_viewport(offset.x, offset.y, size.x, size.y, 0.0, 1.0);

            rpass.set_pipeline(&self.pipelines.render);
            {
                let push_constants = controller.prepare_render(ctx, offset);
                let bytes = bytemuck::bytes_of(&push_constants);
                #[cfg(not(feature = "emulate_constants"))]
                rpass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, bytes);
                #[cfg(feature = "emulate_constants")]
                ctx.queue
                    .write_buffer(&self.emulate_constants_buffer.render, 0, bytes);
            }
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                rpass.set_bind_group(i as u32, bind_group, &[]);
            }
            let (vertices, indices) = controller.get_vertex_index_buffer();
            if let Some((vertex_buffer, num_vertices)) = vertices {
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                if let Some((index_buffer, num_indices)) = indices {
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..num_indices, 0, 0..1);
                } else {
                    rpass.draw(0..num_vertices, 0..1);
                }
            } else {
                rpass.draw(0..3, 0..1);
            }
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn render_ui<C: ControllerTrait>(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &wgpu::TextureView,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut C,
    ) {
        let (clipped_primitives, textures_delta, available_rect, pixels_per_point) =
            ui.prepare(window, ui_state, controller, ctx);

        if available_rect.width() > 0.0 && available_rect.height() > 0.0 {
            self.render_shader(
                ctx,
                output_view,
                controller,
                available_rect * pixels_per_point,
            );
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [ctx.config.width, ctx.config.height],
            pixels_per_point,
        };

        for (id, delta) in &textures_delta.set {
            self.ui_renderer
                .update_texture(&ctx.device, &ctx.queue, *id, delta);
        }

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Encoder"),
            });

        self.ui_renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });

            for id in &textures_delta.free {
                self.ui_renderer.free_texture(id);
            }

            self.ui_renderer.render(
                &mut rpass.forget_lifetime(),
                &clipped_primitives,
                &screen_descriptor,
            );
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    pub fn new_module(&mut self, ctx: &GraphicsContext, shader_path: &std::path::Path) {
        self.pipelines = create_pipelines(
            &ctx.device,
            &self.pipeline_layouts,
            ctx.config.format,
            &self.vertex_buffer_layouts,
            &std::fs::read(shader_path).unwrap(),
        );
    }

    pub fn shader_offset(&self) -> glam::Vec2 {
        glam::vec2(self.shader_viewport.left(), self.shader_viewport.top())
    }
}

fn create_pipeline_layouts(
    ctx: &GraphicsContext,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> PipelineLayouts {
    let create = |push_constant_ranges| {
        ctx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges,
            })
    };
    PipelineLayouts {
        render: create(&[
            #[cfg(not(feature = "emulate_constants"))]
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..128,
            },
        ]),
        #[cfg(feature = "compute")]
        compute: create(&[
            #[cfg(not(feature = "emulate_constants"))]
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..128,
            },
        ]),
    }
}

fn create_pipelines(
    device: &wgpu::Device,
    pipeline_layouts: &PipelineLayouts,
    surface_format: wgpu::TextureFormat,
    vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
    shader_bytes: &[u8],
) -> Pipelines {
    let spirv = wgpu::util::make_spirv(shader_bytes);
    let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: spirv,
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layouts.render),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("main_vs"),
            buffers: vertex_buffer_layouts,
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some("main_fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    });
    #[cfg(feature = "compute")]
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layouts.compute),
        module,
        entry_point: Some("main_cs"),
        compilation_options: Default::default(),
        cache: None,
    });
    Pipelines {
        render: render_pipeline,
        #[cfg(feature = "compute")]
        compute: compute_pipeline,
    }
}

#[cfg(feature = "emulate_constants")]
fn create_emulate_constants_bind_groups(
    device: &wgpu::Device,
) -> (
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
    EmulateConstantsBuffer,
) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            #[cfg(feature = "compute")]
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("emulated push constants layout"),
    });
    let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    use wgpu::util::DeviceExt;
    let fragment_constants_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &[0; 128],
        usage,
    });
    #[cfg(feature = "compute")]
    let compute_constants_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &[0; 128],
        usage,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: fragment_constants_buffer.as_entire_binding(),
            },
            #[cfg(feature = "compute")]
            wgpu::BindGroupEntry {
                binding: 1,
                resource: compute_constants_buffer.as_entire_binding(),
            },
        ],
        label: Some("emulated push constants bind group"),
    });
    (
        layout,
        bind_group,
        EmulateConstantsBuffer {
            render: fragment_constants_buffer,
            #[cfg(feature = "compute")]
            compute: compute_constants_buffer,
        },
    )
}
