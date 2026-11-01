use brot3_idl::Colourer;
use brot3_lib::{
    COMPUTE_SHADER_THREADS,
    data::{FragmentConstants, PointResult},
    ui::{ShaderKey, ShaderVariant},
    wgsl::compute_shader,
};
use easy_cast::{Cast as _, Conv as _};
use glam::{UVec2, UVec3, Vec2};
use wgpu::{Device, Queue};

use super::Queries;
use crate::{MAX_MAX_ITERATIONS, save::LoadSaveErrorInternal};

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
    #[error(transparent)]
    LoadSaveError(#[from] LoadSaveErrorInternal),
    #[error("Failed to map staging buffer")]
    StagingBufferMapFailed,
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

struct InitDeviceResult {
    device: Device,
    queue: Queue,
    timestamps_supported: bool,
}

impl ComputeController {
    /// Creates a new `ComputeController` with the specified render size and number of passes.
    pub fn new(render_size: UVec2, n_passes: u32, key: ShaderKey) -> Result<Self, Error> {
        let InitDeviceResult {
            device,
            queue,
            timestamps_supported,
        } = futures::executor::block_on(Self::init_device(render_size))?;
        let (pixel_buffer, staging_buffer, reference_buffer) =
            Self::create_buffers(&device, render_size);

        let (layouts, bind_groups) =
            crate::shader::describe_bind_groups(&device, &reference_buffer, Some(&pixel_buffer));
        let bind_group_layouts = layouts.iter();

        let pipeline_layout = Self::create_pipeline_layout(
            &device,
            &bind_group_layouts.map(Some).collect::<Vec<_>>(),
        );

        let shader = Self::load_shader(&device, key.variant, key.colourer);
        let pipeline = Self::create_pipeline(&device, &pipeline_layout, &shader);
        let queries = if timestamps_supported {
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
    pub fn run<F: FnMut(&[u8]) -> Result<(), LoadSaveErrorInternal>>(
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
            cpass.set_immediates(0, bytemuck::bytes_of(&constants));
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
        let result;
        {
            if let Ok(slice) = self.staging_buffer.slice(..).get_mapped_range() {
                let rgba = bytemuck::cast_slice::<u8, u32>(&slice);
                log::trace!(
                    "Pixel data: {:08x?}.. ({} bytes)",
                    &rgba[0..10],
                    slice.len()
                );
                result = frame_cb(&slice);
            } else {
                result = Err(LoadSaveErrorInternal::StagingBufferMapFailed);
            }
        }
        self.staging_buffer.unmap();
        result?;

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

    async fn init_device(render_size: UVec2) -> Result<InitDeviceResult, Error> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                flags: wgpu::InstanceFlags::default(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
                backend_options: wgpu::BackendOptions::default(),
                display: None,
            }
            .with_env(),
        );

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (features, limits, timestamps_supported) = Self::describe_wgpu_features_and_limits(
            render_size,
            adapter.features(),
            &adapter.limits(),
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("b3compute"),
                required_features: features,
                required_limits: limits,
                ..Default::default()
            })
            .await?;
        Ok(InitDeviceResult {
            device,
            queue,
            timestamps_supported,
        })
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
        let reference_buffer_size = std::mem::size_of::<Vec2>() * (MAX_MAX_ITERATIONS + 1);
        let reference_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reference_buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            size: reference_buffer_size.cast(),
            mapped_at_creation: false,
        });
        (pixel_buffer, staging_buffer, reference_buffer)
    }

    fn describe_wgpu_features_and_limits(
        render_size: UVec2,
        supported_features: wgpu::Features,
        supported_limits: &wgpu::Limits,
    ) -> (wgpu::Features, wgpu::Limits, bool) {
        use wgpu::Features;

        let max_storage_buffer_binding_size = u64::conv(core::mem::size_of::<PointResult>())
            * u64::from(render_size.element_product()).max(u64::conv(
                std::mem::size_of::<Vec2>() * (MAX_MAX_ITERATIONS + 1),
            ));
        let max_buffer_size = max_storage_buffer_binding_size;

        let mut features = wgpu::Features::default() | wgpu::Features::IMMEDIATES;

        let timestamps_supported = supported_features
            .contains(Features::TIMESTAMP_QUERY | Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        if timestamps_supported {
            features |= Features::TIMESTAMP_QUERY | Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }

        let max_immediate_size = supported_limits.max_immediate_size.max(128);

        (
            features,
            wgpu::Limits {
                max_storage_buffer_binding_size,
                max_buffer_size,
                max_immediate_size,
                ..Default::default()
            },
            timestamps_supported,
        )
    }

    fn load_shader(
        device: &Device,
        variant: ShaderVariant,
        colourer: Colourer,
    ) -> wgpu::ShaderModule {
        let label: &'static str = variant.into();
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&compute_shader(
                ShaderKey { variant, colourer },
            ))),
        })
    }

    fn create_pipeline_layout(
        device: &Device,
        bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("b3compute pipeline"),
            bind_group_layouts,
            immediate_size: 128,
        })
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

    pub(crate) fn get_timestamp_period(&self) -> f64 {
        self.queue.get_timestamp_period().into()
    }
}

#[cfg(test)]
mod tests {
    use brot3_idl::WgslVec2;
    use brot3_lib::{
        data::{Flags, FragmentConstants},
        ui::ShaderVariant,
        util::Size,
    };

    use super::*;

    #[test]
    fn compute_controller_emits_non_black_gpu_output() {
        let render_size = UVec2::new(16, 16);
        let mut controller = ComputeController::new(render_size, 1, ShaderKey::default())
            .expect("compute controller should initialize");

        let mut constants = FragmentConstants::default();
        constants.flags = WgslFlags::empty();
        constants.viewport_size = Size::new(render_size.x, render_size.y);
        constants.buffer_size = Size::new(render_size.x, render_size.y);
        constants.viewport_translate = WgslVec2::ZERO;
        constants.viewport_zoom = 0.25;
        constants.max_iter = 96;

        let mut pixels = Vec::new();
        let _timings = controller
            .run(constants, render_size.extend(1), 1, &[], |rgba| {
                pixels.extend_from_slice(rgba);
                Ok(())
            })
            .expect("compute controller should render successfully");

        let first_pixel = &pixels[..4];
        assert_ne!(
            first_pixel,
            &[0_u8, 0, 0, 0],
            "pixel (0,0) should escape and produce non-black output"
        );
        assert!(
            pixels.iter().any(|&byte| byte != 0),
            "compute output should contain non-zero bytes"
        );
        assert_eq!(pixels.len(), (render_size.x * render_size.y * 4) as usize);
        assert!(!pixels.is_empty());
    }
}
