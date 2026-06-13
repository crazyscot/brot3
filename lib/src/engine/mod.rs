//! Pixel-related calculations (generic by type)
// (c) 2025-6 Ross Younger

#![allow(unused_imports)]

mod colour;
pub(crate) mod entrypoints;
pub(crate) mod fractal;
pub use colour::colour_data;
#[cfg(feature = "all-fractals")]
pub use fractal::AlgorithmModifiers;
#[cfg(feature = "perturbation-mode")]
pub use fractal::MandelbrotPerturbed;
#[cfg(not(spirv))]
pub use fractal::mandelbrot_perturbed_compute_reference_iters;
pub use fractal::render;
// Expose internals for use by benchmarks
#[doc(hidden)]
pub use fractal::{AlgorithmDetail, MandelbrotFamily, RunningConstants, RunningVariables};

use crate::{UVec2, Vec2, uvec2};

/// The size of the complex plane that you see at the default zoom level, at the
/// [`NOMINAL_WINDOW_SIZE`].
pub const DEFAULT_FRACTAL_PLANE_SIZE: f64 = 4.0;

/// The window size that defines a zoom factor of 1.0.
///
/// This happens to be what we get by default from winit on Linux.
pub const NOMINAL_WINDOW_SIZE: UVec2 = uvec2(800, 600);

/// Helper trait for calculating the size of a pixel in the complex plane, given the current zoom
/// and viewport size.
pub trait PixelSpacing
where
    Self: num_traits::Float + 'static + Copy + core::marker::Sized,
{
    /// Get the size of a pixel in the complex plane, given the current zoom and viewport size.
    ///
    /// This may go wrong if the `pixel_count` doesn't convert precisely into the type T,
    /// but that should be fine for our purposes since we only use this for standard displays.
    #[must_use]
    fn pixel_spacing(self, pixel_count: u32) -> Self {
        Self::one() / self.pixel_spacing_inv(pixel_count)
    }

    /// Get the inverse of the pixel spacing, which is more efficient to calculate for display
    /// purposes.
    ///
    /// # Panics
    /// The default implementation panics if the `pixel_count` could not be converted to the type T,
    /// but this should not happen for our use cases.
    #[must_use]
    fn pixel_spacing_inv(self, pixel_count: u32) -> Self {
        Self::from(pixel_count).unwrap() * self
    }
}

impl PixelSpacing for f32 {}

/// This trait impl is not useful on spirv unless your GPU supports it and you declare it as a
/// required feature.
impl PixelSpacing for f64 {}

pub(crate) fn new_york_distance(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
