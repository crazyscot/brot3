//! Helper wrapper for [`wgpu::QuerySet`], for timing GPU operations.

use wgpu::Device;

// Helper wrapper for [`wgpu::QuerySet`], for timing GPU operations.
#[allow(missing_debug_implementations, missing_docs)]
#[derive(derive_more::Debug)]
pub struct Queries {
    #[debug(skip)]
    pub query_set: wgpu::QuerySet,
    #[debug(skip)]
    resolve_buffer: wgpu::Buffer,
    #[debug(skip)]
    destination_buffer: wgpu::Buffer,
    pub size: u32,
    pub next: u32,
}
impl Queries {
    /// Creates a new `Queries` instance for up to the specified number of queries.
    #[must_use]
    pub fn new(device: &Device, count: u32) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("timing query set"),
            ty: wgpu::QueryType::Timestamp,
            count,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("query resolve buffer"),
            size: std::mem::size_of::<u64>() as u64 * u64::from(count),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
            mapped_at_creation: false,
        });
        let destination_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("query destination buffer"),
            size: std::mem::size_of::<u64>() as u64 * u64::from(count),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            query_set,
            resolve_buffer,
            destination_buffer,
            size: count,
            next: 0,
        }
    }

    /// Resolves the queries and copies the results to the destination buffer.
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(
            &self.query_set,
            // TODO(https://github.com/gfx-rs/wgpu/issues/3993): Musn't be larger than the number valid queries in the set.
            0..self.next,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.destination_buffer,
            0,
            self.resolve_buffer.size(),
        );
    }

    /// Blocks until the results are available, then returns the query results as a vector of
    /// timestamps in nanoseconds.
    #[must_use]
    pub fn wait_for_results(&self, device: &wgpu::Device) -> Vec<u64> {
        self.destination_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| ());
        let _ = device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        let timestamps = {
            let timestamp_view = self
                .destination_buffer
                .slice(..(std::mem::size_of::<u64>() as wgpu::BufferAddress * u64::from(self.size)))
                .get_mapped_range();
            bytemuck::allocation::pod_collect_to_vec(&timestamp_view)
        };

        self.destination_buffer.unmap();

        timestamps
    }

    /// Adds a GPU command to the encoder that writes a timestamp to this query set.
    pub fn write_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.next < self.size {
            encoder.write_timestamp(&self.query_set, self.next);
            self.next += 1;
        }
    }
}
