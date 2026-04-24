use brot3_lib::{
    COMPUTE_SHADER_THREADS,
    data::{FragmentConstants, PointResult},
};
use easy_cast::Cast as _;
use glam::{UVec2, UVec3, Vec2};
use wgpu::{Device, Queue, RequestDeviceError, ShaderModule};

use super::Queries;
use crate::MAX_MAX_ITERATIONS;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error(transparent)]
    BufferAsync(#[from] wgpu::BufferAsyncError),
    #[error(transparent)]
    Canceled(#[from] futures_channel::oneshot::Canceled),
    #[error(transparent)]
    PollError(#[from] wgpu::PollError),
    #[error(transparent)]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Shader not present due to suppress-shader-build feature")]
    SuppressedShaderBuild,
}

#[allow(missing_docs)]
#[derive(derive_more::Debug)]
pub struct ComputeController {
    pub render_size: UVec2,
    // TODO Fractal detail
    pixel_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    reference_buffer: wgpu::Buffer,

    device: Device,
    queue: Queue,
    pipeline: wgpu::ComputePipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    queries: Option<Queries>,
}

impl ComputeController {
    /// Creates a new `ComputeController` with the specified render size and number of passes.
    #[cfg(feature = "suppress-shader-build")]
    pub fn new(render_size: UVec2, n_passes: u32, timestamps: bool) -> Result<Self, Error> {
        Err(Error::SuppressedShaderBuild)
    }

    #[cfg(not(feature = "suppress-shader-build"))]
    /// Creates a new `ComputeController` with the specified render size and number of passes.
    pub fn new(render_size: UVec2, n_passes: u32, timestamps: bool) -> Result<Self, Error> {
        let (device, queue) =
            futures::executor::block_on(Self::init_device(render_size, timestamps))?;
        let (pixel_buffer, staging_buffer, reference_buffer) =
            Self::create_buffers(&device, render_size);

        let (layouts, bind_groups) =
            Self::describe_bind_groups(&device, render_size, &pixel_buffer, &reference_buffer);
        let bind_group_layouts = layouts.iter();

        let pipeline_layout =
            Self::create_pipeline_layout(&device, &bind_group_layouts.collect::<Vec<_>>());

        let shader = Self::load_shader(&device);
        let pipeline = Self::create_pipeline(&device, &pipeline_layout, &shader);
        let queries = if timestamps {
            Some(Queries::new(&device, 2 * n_passes + 2))
        } else {
            None
        };

        Ok(Self {
            render_size,
            pixel_buffer,
            staging_buffer,
            reference_buffer,
            device,
            queue,
            pipeline,
            bind_groups,
            queries,
        })
    }

    /// Runs the compute shader for the specified number of passes, with the given constants and
    /// dimensions.
    ///
    /// Number of passes is usually 1, but can be higher for testing and benchmarking purposes.
    /// The callback will be called once per pass, with the raw RGBA pixel data for that pass.
    // N.B. Here, we diverge significantly from easy-shader-runner's pattern.
    // ESR is designed to run a compute dispatch once per frame, hand in hand with winit window.
    // This compute shader is independent of a window.
    // If it is desirable to refactor ESR so we can use it, getting the API right will be
    // non-trivial. (Pipeline setup looks fairly similar and straightforward to align.)
    pub fn run<F: FnMut(&[u8])>(
        &mut self,
        constants: FragmentConstants,
        dimensions: UVec3,
        n_passes: u32,
        reference_points: &[Vec2],
        mut frame_cb: F,
    ) -> Result<Vec<u64>, Error> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // manual impl of ceil(dimensions / COMPUTE_SHADER_THREADS)
        let workspace = (dimensions + COMPUTE_SHADER_THREADS - UVec3::ONE) / COMPUTE_SHADER_THREADS;

        if let Some(queries) = &mut self.queries {
            queries.write_timestamp(&mut encoder);
        }
        for _pass in 0..n_passes {
            let timestamp_writes = if let Some(queries) = self.queries.as_mut() {
                queries.next += 2;
                Some(wgpu::ComputePassTimestampWrites {
                    query_set: &queries.query_set,
                    beginning_of_pass_write_index: Some(queries.next - 2),
                    end_of_pass_write_index: Some(queries.next - 1),
                })
            } else {
                None
            };
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_push_constants(0, bytemuck::bytes_of(&constants));

            self.queue.write_buffer(
                &self.reference_buffer,
                0,
                bytemuck::cast_slice(reference_points),
            );
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                cpass.set_bind_group(i.cast(), bind_group, &[]);
            }

            cpass.dispatch_workgroups(workspace.x, workspace.y, workspace.z);
            drop(cpass);

            // Copy the results from the GPU buffer to the staging buffer
            encoder.copy_buffer_to_buffer(
                &self.pixel_buffer,
                0,
                &self.staging_buffer,
                0,
                self.this_pixel_buffer_size(),
            );
            // FUTURE: We may be able to streamline this, by having two staging buffers and
            // alternating them.
        }

        if let Some(queries) = self.queries.as_mut() {
            queries.write_timestamp(&mut encoder);
            queries.resolve(&mut encoder);
        }
        let _ = self.queue.submit(Some(encoder.finish()));

        let (sender, receiver) = futures_channel::oneshot::channel();
        // Call map_async on the staging buffer with MapMode::Read.
        self.staging_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |result| {
                sender.send(result).unwrap();
            });
        // wait for the result
        let st = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;
        assert!(st.wait_finished());
        assert!(st.is_queue_empty());

        futures::executor::block_on(receiver)??;

        // now we can access the result
        {
            let slice: &[u8] = &self.staging_buffer.slice(..).get_mapped_range();
            let rgba = bytemuck::cast_slice::<u8, u32>(slice);
            log::trace!(
                "Pixel data: {:08x?}.. ({} bytes)",
                &rgba[0..10],
                slice.len()
            );
            frame_cb(slice);
        }
        self.staging_buffer.unmap();

        // Extract timing info from GPU
        if let Some(queries) = self.queries.as_mut() {
            Ok(queries.wait_for_results(&self.device))
        } else {
            Ok(vec![])
        }
    }

    fn this_pixel_buffer_size(&self) -> u64 {
        Self::pixel_buffer_size_generic(self.render_size)
    }

    fn pixel_buffer_size_generic(render_size: UVec2) -> u64 {
        std::mem::size_of::<f32>() as u64 * u64::from(render_size.element_product())
    }

    async fn init_device(
        render_size: UVec2,
        timestamps: bool,
    ) -> Result<(Device, Queue), RequestDeviceError> {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                flags: wgpu::InstanceFlags::default(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
                backend_options: wgpu::BackendOptions::default(),
            }
            .with_env(),
        );

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (features, limits) = Self::describe_wgpu_features_and_limits(
            render_size,
            adapter.features(),
            &adapter.limits(),
        );
        let (mut features, limits) = if cfg!(feature = "emulate_constants") {
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
        if timestamps {
            features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("b3compute"),
                required_features: features,
                required_limits: limits,
                ..Default::default()
            })
            .await
    }

    fn create_buffers(
        device: &Device,
        render_size: UVec2,
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        let pixel_buffer_size = Self::pixel_buffer_size_generic(render_size);
        let pixel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pixel_buffer"),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            size: pixel_buffer_size,
            mapped_at_creation: false,
        });
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            size: pixel_buffer_size,
            mapped_at_creation: false,
        });
        let reference_buffer_size =
            std::mem::size_of::<Vec2>() as u64 * u64::from(MAX_MAX_ITERATIONS + 1);
        let reference_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reference_buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            size: reference_buffer_size,
            mapped_at_creation: false,
        });
        (pixel_buffer, staging_buffer, reference_buffer)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn describe_wgpu_features_and_limits(
        render_size: UVec2,
        _supported_features: wgpu::Features,
        supported_limits: &wgpu::Limits,
    ) -> (wgpu::Features, wgpu::Limits) {
        let max_storage_buffer_binding_size = core::mem::size_of::<PointResult>() as u32
            * render_size
                .element_product()
                .max(std::mem::size_of::<Vec2>() as u32 * (MAX_MAX_ITERATIONS + 1));
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
        device: &Device,
        render_size: UVec2,
        pixel_buffer: &wgpu::Buffer,
        reference_buffer: &wgpu::Buffer,
    ) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // Pixel buffer (output)
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // Perturbation reference buffer (input)
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // emulate_constants would go here if we needed it
            ],
            label: Some("b3compute bgl"),
        });

        assert!(render_size != UVec2::ZERO);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pixel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: reference_buffer.as_entire_binding(),
                },
            ],
            label: Some("fractal_bind_group"),
        });
        (vec![layout], vec![bind_group])
    }

    #[cfg(not(feature = "suppress-shader-build"))]
    fn load_shader(device: &Device) -> ShaderModule {
        let spirv = wgpu::util::make_spirv(crate::SHADER_BYTES);
        let label = "b3compute";
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: spirv,
        })
    }

    fn create_pipeline_layout(
        device: &Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let create = |push_constant_ranges| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("b3compute pipeline"),
                bind_group_layouts,
                push_constant_ranges,
            })
        };
        create(&[
            #[cfg(not(feature = "emulate_constants"))]
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..128,
            },
        ])
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
    ) -> wgpu::ComputePipeline {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("b3compute"),
            layout: Some(pipeline_layout),
            module,
            entry_point: Some("main_cs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
