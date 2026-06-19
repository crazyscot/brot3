use easy_cast::ConvApprox as _;
use egui_winit::winit::window::Window;
use num_traits::AsPrimitive as _;
use wgpu::{CurrentSurfaceTexture, PipelineCompilationOptions};

use crate::{
    Error, ShaderDescriptor, ShaderSource,
    context::GraphicsContext,
    controller::ControllerTrait,
    ui::{Ui, UiState},
};

// Minimal fullscreen blit WGSL moved to a const to keep function bodies short for clippy
const BLIT_WGSL: &str = r"
struct VSOutput {
    @builtin(position) Position: vec4<f32>,
    @location(0) fragUV: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex: u32) -> VSOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );
    var out: VSOutput;
    out.Position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    out.fragUV = (out.Position.xy * 0.5) + vec2<f32>(0.5, 0.5);
    // flip Y so texture sampling matches UI coordinate space
    out.fragUV.y = 1.0 - out.fragUV.y;
    return out;
}

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var tex: texture_2d<f32>;

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.fragUV);
}
";

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

pub(crate) struct RenderPass {
    pipelines: Pipelines,
    pipeline_layouts: PipelineLayouts,
    shaders: Vec<ShaderDescriptor>,
    active_shader_key: &'static str,
    default_shader_key: &'static str,
    ui_renderer: egui_wgpu::Renderer,
    bind_groups: Vec<wgpu::BindGroup>,
    shader_viewport: egui::Rect,
    #[cfg(feature = "emulate_constants")]
    emulate_constants_buffer: EmulateConstantsBuffer,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    query_set: Option<wgpu::QuerySet>,
    resolve_buffer: Option<wgpu::Buffer>,
    destination_buffer: Option<wgpu::Buffer>,
    destination_buffer_size: u64,

    // Ping-pong offscreen render targets
    ping_textures: Option<[wgpu::Texture; 2]>,
    ping_views: Option<[wgpu::TextureView; 2]>,
    current_ping: usize,
    offscreen_size: (u32, u32),

    // Blit pipeline to present an offscreen texture to the swapchain
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    blit_sampler: wgpu::Sampler,

    pub(crate) last_elapsed: Option<std::time::Duration>,
}

impl RenderPass {
    fn create_blit_resources(
        ctx: &GraphicsContext,
    ) -> (wgpu::Sampler, wgpu::BindGroupLayout, wgpu::RenderPipeline) {
        let blit_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("blit-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let blit_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("blit-bind-group-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let blit_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("blit-pipeline-layout"),
                    bind_group_layouts: &[Some(&blit_bind_group_layout)],
                    #[cfg(not(feature = "emulate_constants"))]
                    immediate_size: 0,
                    #[cfg(feature = "emulate_constants")]
                    immediate_size: 0,
                });

        let blit_shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("blit-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLIT_WGSL)),
            });

        let blit_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("blit-pipeline"),
                layout: Some(&blit_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &blit_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: PipelineCompilationOptions::default(),
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
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &blit_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                multiview_mask: None,
                cache: None,
            });

        (blit_sampler, blit_bind_group_layout, blit_pipeline)
    }
}

impl RenderPass {
    pub(crate) fn new<C: ControllerTrait>(
        ctx: &GraphicsContext,
        shaders: Vec<ShaderDescriptor>,
        default_shader_key: &'static str,
        controller: &mut C,
    ) -> Result<Self, Error> {
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
            create_pipeline_layouts(ctx, &bind_group_layouts.map(Some).collect::<Vec<_>>());
        let active_shader_key = controller
            .current_shader_key()
            .unwrap_or(default_shader_key);
        let pipelines = create_pipelines(
            &ctx.device,
            &pipeline_layouts,
            ctx.config.format,
            &vertex_buffer_layouts,
            load_shader_bytes(find_shader(&shaders, active_shader_key)?)?.as_ref(),
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

        // Create blit resources (sampler, bind group layout and fullscreen pipeline) via helper
        let (blit_sampler, blit_bind_group_layout, blit_pipeline) =
            Self::create_blit_resources(ctx);

        let query_set;
        let resolve_buffer;
        let destination_buffer;
        let destination_buffer_size;
        if ctx.timestamps {
            destination_buffer_size = 16; // 2 timestamps * 8 bytes (u64)
            query_set = Some(ctx.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("benchmark-query-set"),
                ty: wgpu::QueryType::Timestamp,
                count: 2, // 1 for start, 1 for end
            }));
            resolve_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query-resolve-buffer"),
                size: destination_buffer_size,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }));
            destination_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query-read-buffer"),
                size: destination_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
        } else {
            query_set = None;
            resolve_buffer = None;
            destination_buffer = None;
            destination_buffer_size = 0;
        }

        // Create two offscreen textures for ping-pong via helper
        let offscreen_size = (ctx.config.width, ctx.config.height);
        let (ping_textures, ping_views) = Self::create_offscreen_textures(ctx, offscreen_size);

        Ok(Self {
            pipelines,
            pipeline_layouts,
            shaders,
            active_shader_key,
            default_shader_key,
            ui_renderer,
            bind_groups,
            shader_viewport: egui::Rect::NAN,
            #[cfg(feature = "emulate_constants")]
            emulate_constants_buffer,
            vertex_buffer_layouts,
            query_set,
            resolve_buffer,
            destination_buffer,
            destination_buffer_size,

            // Ping-pong targets
            ping_textures: Some(ping_textures),
            ping_views: Some(ping_views),
            current_ping: 0,
            offscreen_size,

            blit_pipeline,
            blit_bind_group_layout,
            blit_sampler,

            last_elapsed: None,
        })
    }

    #[cfg(feature = "compute")]
    pub(crate) fn compute(
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
                cpass.set_immediates(0, push_constants);
                #[cfg(feature = "emulate_constants")]
                ctx.queue
                    .write_buffer(&self.emulate_constants_buffer.compute, 0, push_constants);
            }
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                cpass.set_bind_group(i as u32, bind_group, &[]);
            }
            cpass.dispatch_workgroups(workspace.x, workspace.y, workspace.z);
        }
        let _ = ctx.queue.submit(Some(encoder.finish()));
    }

    pub(crate) fn render<C: ControllerTrait>(
        &mut self,
        ctx: &mut GraphicsContext,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut C,
    ) -> bool {
        // Deal with the previous timestamps, if any
        if let Some(destination_buffer) = &self.destination_buffer {
            destination_buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, |_| ());
            let _ = ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .unwrap();
            let timestamp_view = destination_buffer
                .slice(..self.destination_buffer_size)
                .get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&timestamp_view);
            let ticks = timestamps[1].saturating_sub(timestamps[0]);
            let resolution = ctx.queue.get_timestamp_period();
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let nanos = (ticks as f64 * f64::from(resolution)) as u64;
            drop(timestamp_view);
            destination_buffer.unmap();
            self.last_elapsed = Some(std::time::Duration::from_nanos(nanos));
        }

        let state = ctx.surface.get_current_texture();
        let texture = match state {
            CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Suboptimal(texture) => {
                log::info!("wgpu surface is suboptimal, reconfiguring");
                ctx.surface.configure(&ctx.device, &ctx.config);
                texture
            }
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => return true,

            CurrentSurfaceTexture::Outdated => {
                ctx.surface.configure(&ctx.device, &ctx.config);
                log::info!("wgpu surface is outdated, reconfiguring");
                return true;
            }

            CurrentSurfaceTexture::Lost => {
                log::error!("wgpu Surface was lost");
                // SOMEDAY: We might recreate the surface (or the whole Device) here. For now, just
                // exit.
                return false;
            }
            CurrentSurfaceTexture::Validation => {
                // SOMEDAY: We might attempt to deal with the validation error here. For now, just
                // exit.
                log::error!("wgpu Surface raised a Validation error");
                return false;
            }
        };

        let output_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Recreate offscreen textures if surface size changed. This has to happen before the render
        // pass.
        if self.ping_views.is_some() && (ctx.config.width, ctx.config.height) != self.offscreen_size
        {
            self.recreate_offscreen(ctx);
            ctx.invalid = true;
        }
        if !self.render_ui(ctx, &output_view, window, ui, ui_state, controller) {
            return false;
        }
        texture.present();
        true
    }

    fn render_shader<C: ControllerTrait>(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &wgpu::TextureView,
        controller: &mut C,
        available_rect: egui::Rect,
        target_ping: Option<usize>,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shader Encoder"),
            });
        {
            let timestamp_writes = if ctx.timestamps {
                Some(wgpu::RenderPassTimestampWrites {
                    query_set: self.query_set.as_ref().unwrap(),
                    beginning_of_pass_write_index: Some(0), // Slot 0
                    end_of_pass_write_index: Some(1),       // Slot 1
                })
            } else {
                None
            };
            // Choose the render target: either an offscreen ping view (if requested) or the
            // provided output view
            let chosen_view: &wgpu::TextureView = if let Some(idx) = target_ping {
                &self.ping_views.as_ref().unwrap()[idx]
            } else {
                output_view
            };

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shader Render Pass"),
                occlusion_query_set: None,
                timestamp_writes,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: chosen_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
            });

            let size = glam::vec2(available_rect.width(), available_rect.height()).floor();
            if self.shader_viewport != available_rect {
                self.shader_viewport = available_rect;
                controller.resize(size.as_uvec2());
            }
            let offset = self.shader_offset();
            // Use the shader offset for the viewport so the shader draws to the correct
            // region of the offscreen texture when ping-ponging. This maps the controller's
            // coordinates correctly into the offscreen target.
            // wgpu viewport Y origin is top-left; convert from UI coords (top-based) to
            // wgpu viewport coordinates so dragging and mouse math remain consistent.

            let vp_y = f32::conv_approx(ctx.config.height) - offset.y - size.y;
            rpass.set_viewport(offset.x, vp_y, size.x, size.y, 0.0, 1.0);

            rpass.set_pipeline(&self.pipelines.render);
            {
                let push_constants = controller.prepare_render(ctx, offset);
                let bytes = bytemuck::bytes_of(&push_constants);
                #[cfg(not(feature = "emulate_constants"))]
                rpass.set_immediates(0, bytes);
                #[cfg(feature = "emulate_constants")]
                ctx.queue
                    .write_buffer(&self.emulate_constants_buffer.render, 0, bytes);
            }
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                rpass.set_bind_group(i.as_(), bind_group, &[]);
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
        if ctx.timestamps {
            encoder.resolve_query_set(
                self.query_set.as_ref().unwrap(),
                0..2, // Resolve both the start and end timestamps
                self.resolve_buffer.as_ref().unwrap(),
                0,
            );
            encoder.copy_buffer_to_buffer(
                self.resolve_buffer.as_ref().unwrap(),
                0,
                self.destination_buffer.as_ref().unwrap(),
                0,
                16,
            );
        }

        let _ = ctx.queue.submit(Some(encoder.finish()));
    }

    fn render_suppressed(ui_state: &UiState, ctx: &GraphicsContext) -> bool {
        ui_state.suppress_render && !ctx.invalid
    }

    fn render_ui<C: ControllerTrait>(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &wgpu::TextureView,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut C,
    ) -> bool {
        let (clipped_primitives, textures_delta, available_rect, pixels_per_point) =
            ui.prepare(window, ui_state, controller, ctx);

        let suppress = Self::render_suppressed(ui_state, ctx);

        if available_rect.width() > 0.0 && available_rect.height() > 0.0 {
            let shader_key = controller
                .current_shader_key()
                .unwrap_or(self.default_shader_key);
            if let Err(error) = self.select_shader(ctx, shader_key) {
                log::error!("{error}");
                return false;
            }
            // Render shader into the current offscreen target (if available) or directly to the
            // output. If suppress_render is set, skip rendering so the previous texture remains
            // unchanged and can be held for presentation.
            let target_ping = if self.ping_views.is_some() {
                Some(self.current_ping)
            } else {
                None
            };
            if !suppress {
                self.render_shader(
                    ctx,
                    output_view,
                    controller,
                    available_rect * pixels_per_point,
                    target_ping,
                );
            }
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [ctx.config.width, ctx.config.height],
            pixels_per_point,
        };

        self.finish_frame(
            ctx,
            output_view,
            &clipped_primitives,
            &textures_delta,
            &screen_descriptor,
            ui_state,
        )
    }

    fn finish_frame(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &wgpu::TextureView,
        clipped_primitives: &[egui::epaint::ClippedPrimitive],
        textures_delta: &egui::epaint::textures::TexturesDelta,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        ui_state: &UiState,
    ) -> bool {
        for (id, delta) in &textures_delta.set {
            self.ui_renderer
                .update_texture(&ctx.device, &ctx.queue, *id, delta);
        }

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Encoder"),
            });

        let _ = self.ui_renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            clipped_primitives,
            screen_descriptor,
        );

        let suppress_render = Self::render_suppressed(ui_state, ctx);

        // Blit the chosen offscreen texture (current or previous) to the swapchain output before UI
        // draws
        if let Some(views) = &self.ping_views {
            let source_index = if suppress_render {
                (self.current_ping + 1) % 2
            } else {
                self.current_ping
            };
            let source_view = &views[source_index];
            let blit_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.blit_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(source_view),
                    },
                ],
                label: Some("blit-bind-group"),
            });

            let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Render Pass"),
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
                multiview_mask: None,
            });
            blit_pass.set_pipeline(&self.blit_pipeline);
            blit_pass.set_bind_group(0, &blit_bind_group, &[]);
            // Draw two triangles (6 vertices) to cover the full screen
            blit_pass.draw(0..6, 0..1);
        }

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
                multiview_mask: None,
            });

            for id in &textures_delta.free {
                self.ui_renderer.free_texture(id);
            }

            self.ui_renderer.render(
                &mut rpass.forget_lifetime(),
                clipped_primitives,
                screen_descriptor,
            );
        }

        // Advance ping index so next frame writes into the other texture, but only when not
        // suppressed. When suppressed, we hold the previous texture contents.
        if self.ping_views.is_some() && !suppress_render {
            self.current_ping = (self.current_ping + 1) % 2;
        }

        let _ = ctx.queue.submit(Some(encoder.finish()));
        true
    }

    pub(crate) fn select_shader(
        &mut self,
        ctx: &GraphicsContext,
        shader_key: &'static str,
    ) -> Result<(), Error> {
        if shader_key == self.active_shader_key {
            return Ok(());
        }
        match self.rebuild_pipelines(ctx, shader_key) {
            Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }

    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    pub(crate) fn new_module(
        &mut self,
        ctx: &GraphicsContext,
        shader_key: &'static str,
        shader_path: &std::path::Path,
    ) -> Result<bool, Error> {
        let shader = find_shader_mut(&mut self.shaders, shader_key)?;
        shader.source = ShaderSource::RuntimePath(shader_path.to_path_buf());
        if shader_key == self.active_shader_key {
            self.rebuild_pipelines(ctx, shader_key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn shader_offset(&self) -> glam::Vec2 {
        glam::vec2(self.shader_viewport.left(), self.shader_viewport.top())
    }

    fn recreate_offscreen(&mut self, ctx: &GraphicsContext) {
        let size = (ctx.config.width, ctx.config.height);
        let (textures, views) = Self::create_offscreen_textures(ctx, size);
        self.ping_textures = Some(textures);
        self.ping_views = Some(views);
        self.offscreen_size = size;
        self.current_ping %= 2;
    }

    fn rebuild_pipelines(
        &mut self,
        ctx: &GraphicsContext,
        shader_key: &'static str,
    ) -> Result<(), Error> {
        let shader_bytes = load_shader_bytes(find_shader(&self.shaders, shader_key)?)?;
        self.pipelines = create_pipelines(
            &ctx.device,
            &self.pipeline_layouts,
            ctx.config.format,
            &self.vertex_buffer_layouts,
            shader_bytes.as_ref(),
        );
        self.active_shader_key = shader_key;
        Ok(())
    }

    fn create_offscreen_textures(
        ctx: &GraphicsContext,
        size: (u32, u32),
    ) -> ([wgpu::Texture; 2], [wgpu::TextureView; 2]) {
        let tex0_desc = wgpu::TextureDescriptor {
            label: Some("ping-texture-0"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ctx.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let tex1_desc = wgpu::TextureDescriptor {
            label: Some("ping-texture-1"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ctx.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let textures = [
            ctx.device.create_texture(&tex0_desc),
            ctx.device.create_texture(&tex1_desc),
        ];
        let views = [
            textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ];
        (textures, views)
    }
}

fn find_shader<'a>(
    shaders: &'a [ShaderDescriptor],
    shader_key: &'static str,
) -> Result<&'a ShaderDescriptor, Error> {
    shaders
        .iter()
        .find(|shader| shader.key == shader_key)
        .ok_or(Error::UnknownShaderKey(shader_key))
}

#[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
fn find_shader_mut<'a>(
    shaders: &'a mut [ShaderDescriptor],
    shader_key: &'static str,
) -> Result<&'a mut ShaderDescriptor, Error> {
    shaders
        .iter_mut()
        .find(|shader| shader.key == shader_key)
        .ok_or(Error::UnknownShaderKey(shader_key))
}

fn load_shader_bytes(shader: &ShaderDescriptor) -> Result<std::borrow::Cow<'_, [u8]>, Error> {
    match &shader.source {
        ShaderSource::Prebuilt(bytes) => Ok(std::borrow::Cow::Borrowed(*bytes)),
        #[cfg(all(
            any(feature = "runtime-compilation", feature = "hot-reload-shader"),
            not(target_arch = "wasm32")
        ))]
        ShaderSource::RuntimePath(path) => Ok(std::borrow::Cow::Owned(std::fs::read(path)?)),
    }
}

fn create_pipeline_layouts(
    ctx: &GraphicsContext,
    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
) -> PipelineLayouts {
    let create = || {
        ctx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                #[cfg(not(feature = "emulate_constants"))]
                immediate_size: 128,
                #[cfg(feature = "emulate_constants")]
                immediate_size: 0,
            })
    };
    PipelineLayouts {
        render: create(),
        #[cfg(feature = "compute")]
        compute: create(),
    }
}

fn create_pipelines(
    device: &wgpu::Device,
    pipeline_layouts: &PipelineLayouts,
    surface_format: wgpu::TextureFormat,
    vertex_buffer_layouts: &[wgpu::VertexBufferLayout<'_>],
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
            compilation_options: PipelineCompilationOptions::default(),
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
            compilation_options: PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    });
    #[cfg(feature = "compute")]
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layouts.compute),
        module,
        entry_point: Some("main_cs"),
        compilation_options: PipelineCompilationOptions::default(),
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
    use wgpu::util::DeviceExt;
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
