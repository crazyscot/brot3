//! Helper types for GPU shaders

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! ## Feature flags
#![doc = document_features::document_features!()]

pub mod grid;
pub use grid::*;

#[cfg(not(target_arch = "spirv"))]
/// Re-exported from [`glam`].
pub use glam::{uvec2, vec2, DVec2, UVec2, Vec2, Vec3};

#[cfg(target_arch = "spirv")]
/// Re-exported from [`glam`].
pub use spirv_std::glam::{uvec2, vec2, DVec2, UVec2, Vec2, Vec3};

/// Arbitrary precision versions of `Complex` and `Vec2`.
/// **Only available on non-GPU builds** and gated by the `big` feature flag.
#[cfg(all(feature = "big", not(target_arch = "spirv")))]
pub mod big {
    pub use super::big_complex::BigComplex;
    pub use super::big_vec2::BigVec2;
}

pub mod colourspace;

use bytemuck::NoUninit;

/// GPU-friendly representation of a two-dimensional `u32` vector
///
#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
#[allow(missing_docs)] // self-explanatory !
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Constructor
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Calculates the aspect ratio
    ///
    /// ```
    /// # use shader_util::Size;
    /// let sz = Size::new(100, 200);
    /// assert_eq!(sz.aspect_ratio(), 0.5);
    /// ```
    #[must_use]
    pub fn aspect_ratio(self) -> f32 {
        #![allow(clippy::cast_precision_loss)]
        self.width as f32 / self.height as f32
    }

    /// Converts to a [`Vec2`]
    ///
    /// ```
    /// # use shader_util::Size;
    /// let sz = Size::new(100,200);
    /// let v = sz.as_vec2();
    /// assert_eq!(v.x, 100.0);
    /// assert_eq!(v.y, 200.0);
    /// ```
    #[must_use]
    pub fn as_vec2(self) -> Vec2 {
        #![allow(clippy::cast_precision_loss)]
        vec2(self.width as f32, self.height as f32)
    }

    /// Converts to a [`UVec2`]
    /// ```
    /// # use shader_util::Size;
    /// let sz = Size::new(100,200);
    /// let v = sz.as_uvec2();
    /// assert_eq!(v.x, 100);
    /// assert_eq!(v.y, 200);
    /// ```
    #[must_use]
    pub fn as_uvec2(self) -> UVec2 {
        uvec2(self.width, self.height)
    }
}

impl From<UVec2> for Size {
    /// ```
    /// # use shader_util::Size;
    /// # use glam::uvec2;
    /// let uv = uvec2(200, 100);
    /// let sz: Size = uv.into();
    /// assert_eq!(sz.aspect_ratio(), 2.0);
    /// ```
    fn from(v: UVec2) -> Self {
        Self {
            width: v.x,
            height: v.y,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::Size;
    use float_eq::assert_float_eq;

    #[test]
    fn conversion() {
        let sz = Size::new(100, 200);
        let v = sz.as_uvec2();
        assert_eq!(v.x, 100);
        assert_eq!(v.y, 200);

        let v = sz.as_vec2();
        assert_float_eq!(v.x, 100.0, ulps <= 4);
        assert_float_eq!(v.y, 200.0, ulps <= 4);
    }

    #[test]
    fn aspect_ratio() {
        let sz = Size::new(100, 200);
        assert_float_eq!(sz.aspect_ratio(), 0.5, ulps <= 4);
    }
}
