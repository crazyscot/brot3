//! Fractal data structures

use bytemuck::NoUninit;

use crate::enums::ColourStyle;

/// Raw data from a fractal invocation
#[derive(Copy, Clone, Debug, Default, NoUninit, derive_more::Constructor)]
#[repr(C)]
pub struct PointResult {
    /// iteration count
    iters: u32,
    /// fractional part of iteration count (range 0..1)
    iters_fraction: f32,
    /// distance estimate from fractal
    distance: f32,
    /// final angle (argument) (range -pi..pi)
    pub angle: f32,
    /// final complex distance, squared
    pub radius_sqr: f32,
}

impl const_default::ConstDefault for PointResult {
    const DEFAULT: Self = Self {
        iters: u32::MAX,
        iters_fraction: 0.0,
        distance: 0.0,
        angle: 0.0,
        radius_sqr: 0.0,
    };
}

impl PointResult {
    // ACCESSORS ////////////////////////////////////////////////////////////
    /// Iterations
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn iters(&self, style: ColourStyle) -> f32 {
        match style {
            ColourStyle::Discrete => self.iters_whole() as f32,
            ColourStyle::Continuous => self.iters_whole() as f32 + self.iters_fraction(),
        }
    }

    /// Whole part of iterations
    #[must_use]
    pub fn iters_whole(&self) -> u32 {
        self.iters
    }

    /// Fractional part of iterations (0..1)
    #[must_use]
    pub fn iters_fraction(&self) -> f32 {
        self.iters_fraction
    }

    /// Distance from fractal
    #[must_use]
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Final angle (-pi .. pi)
    #[must_use]
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Final distance from origin (aka radius or absolute value), squared
    #[must_use]
    pub fn radius_sqr(&self) -> f32 {
        self.radius_sqr
    }

    // COMPUTED ACCESSORS ///////////////////////////////////////////////////
    /// Is this point inside the set? If so, the iterations count is effectively infinite.
    #[must_use]
    pub fn inside(&self) -> bool {
        self.iters == u32::MAX
    }

    /// Debug checker
    ///
    /// # Panics
    /// If any of the checked conditions are not met
    #[cfg(not(target_arch = "spirv"))]
    pub fn assert_no_subnormals(&self) {
        assert!(self.iters_fraction().is_finite());
        assert!(self.distance().is_finite());
        assert!(self.angle().is_finite());
        assert!(self.radius_sqr().is_finite());
    }
}
