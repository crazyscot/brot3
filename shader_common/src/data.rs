//! Fractal data structures

use super::GRID_SIZE;
use bytemuck::NoUninit;

/// Raw data from a fractal invocation
///
/// This structure is split into two sub-structs because of the 128MB default limit on data sizes.
/// With a GRID_SIZE of 3840x2160, the default limit allows us 16.18 bytes per grid pixel.
/// Therefore, split our data into shards, each of which is up to 16 bytes in size.
/// If somehow we need to make GRID_SIZE larger, might need to refactor this to split it differently.
///
/// (Yes, we could check the operational capabilities and request more... but that would involve
/// making things dynamic. Not for today.)
#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResult {
    a: PointResultA,
    b: PointResultB,
}

/// Constituent part A of `PointResult`
#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResultA {
    /// iteration count
    iters: u32,
    /// fractional part of iteration count (range 0..1)
    iters_fraction: f32,
    /// distance estimate from fractal
    distance: f32,
    /// final angle (argument) (range -pi..pi)
    pub angle: f32,
}

/// Constituent part B of `PointResult`
#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct PointResultB {
    /// final complex distance, squared
    pub radius_sqr: f32,
}

// compile time assertion: confirm that neither buffer will runtime fail in wgpu
const _: () = {
    const N_POINTS: usize = (GRID_SIZE.x * GRID_SIZE.y) as usize;
    const LIMIT: usize = 128 * 1024 * 1024; // == wgpu::Limits::max_storage_buffer_binding_size
    assert!(core::mem::size_of::<PointResultA>() * N_POINTS < LIMIT);
    assert!(core::mem::size_of::<PointResultB>() * N_POINTS < LIMIT);
};

impl PointResult {
    // CONSTRUCTORS //////////////////////////////////////////////////////////
    pub fn new_inside(distance: f32, angle: f32, radius_sqr: f32) -> Self {
        Self {
            a: PointResultA {
                iters: u32::MAX,
                iters_fraction: 0.,
                distance,
                angle,
            },
            b: PointResultB { radius_sqr },
        }
    }
    pub fn new_outside(
        iters: u32,
        iters_fraction: f32,
        distance: f32,
        angle: f32,
        radius_sqr: f32,
    ) -> Self {
        Self {
            a: PointResultA {
                iters,
                iters_fraction,
                distance,
                angle,
            },
            b: PointResultB { radius_sqr },
        }
    }
    /// Reconstitutes a `PointResult` from its storage shards
    pub fn join(a: PointResultA, b: PointResultB) -> Self {
        Self { a, b }
    }
    // ACCESSORS ////////////////////////////////////////////////////////////
    pub fn a(&self) -> PointResultA {
        self.a
    }
    pub fn b(&self) -> PointResultB {
        self.b
    }
    /// Iterations
    pub fn iters(&self) -> u32 {
        self.a.iters
    }
    /// Fractional part of iterations (0..1)
    pub fn iters_fraction(&self) -> f32 {
        self.a.iters_fraction
    }
    /// Distance from fractal
    pub fn distance(&self) -> f32 {
        self.a.distance
    }
    /// Final angle
    pub fn angle(&self) -> f32 {
        self.a.angle
    }
    /// Final distance from origin (aka radius or absolute value), squared
    pub fn radius_sqr(&self) -> f32 {
        self.b.radius_sqr
    }
    // COMPUTED ACCESSORS ///////////////////////////////////////////////////
    /// Is this point inside the set? If so, the iterations count is effectively infinite.
    pub fn inside(&self) -> bool {
        self.a.iters == u32::MAX
    }
}
