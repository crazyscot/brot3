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

pub(crate) fn new_york_distance(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
