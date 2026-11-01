//! Interface to shader management

pub(crate) fn describe_bind_groups(
    device: &wgpu::Device,
    reference_buffer: &wgpu::Buffer,
    pixel_buffer: Option<&wgpu::Buffer>,
) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
    // group 0: perturbation data
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("b3.fractal"),
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: reference_buffer.as_entire_binding(),
        }],
        label: Some("b3.fractal"),
    });

    let mut layouts = vec![layout];
    let mut groups = vec![bind_group];

    if let Some(pixel_buffer) = pixel_buffer {
        // group 1 (main_cs only): output pixel data
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("b3.compute"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: pixel_buffer.as_entire_binding(),
            }],
            label: Some("b3.compute"),
        });
        layouts.push(layout);
        groups.push(bind_group);
    }
    (layouts, groups)
}
