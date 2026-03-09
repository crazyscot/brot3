//! Pixel-related calculations (generic by type)
// (c) 2025-6 Ross Younger

mod colour;
pub(crate) mod entrypoints;
pub mod fractal;
pub use colour::colour_data;
pub use fractal::render;

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
