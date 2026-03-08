//! Fractal data structures

use bytemuck::NoUninit;

use crate::{enums::ColourStyle, fractal::BoundaryClass};

/// Raw data from a fractal invocation
#[derive(Copy, Clone, Debug, Default, NoUninit, derive_more::Constructor)]
#[repr(C)]
pub struct PointResult {
    /// iteration count
    iters: u32,
    /// fractional part of iteration count (range 0..1)
    iters_fraction: f32,
    /// final angle (argument) (range -pi..pi)
    pub angle: f32,
    /// final complex distance, squared
    pub radius_sqr: f32,
    /// Is this point considered to be on the boundary?
    pub boundary: BoundaryClass,
}

impl const_default::ConstDefault for PointResult {
    const DEFAULT: Self = Self {
        iters: u32::MAX,
        iters_fraction: 0.0,
        angle: 0.0,
        radius_sqr: 0.0,
        boundary: BoundaryClass::Indeterminate,
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
        assert!(self.angle().is_finite());
        assert!(self.radius_sqr().is_finite());
    }

    // MUTATORS //////////////////////////////////////////////////////////////

    /// Many years ago, my previous fractal plotter `brot2` had a bug where
    /// the number of iterations was incorrectly clamped between passes.
    ///
    /// The effect was serendipitous: it increased the colour contrast and gradient
    /// in some regions (close to the set, where the number of iterations > 256:
    /// typically found at zooms of factor 1000x or more).
    ///
    /// This function reimplements that effect.
    pub fn cull_iterations(&mut self) {
        if self.iters == u32::MAX {
            return;
        }

        let mut passcount = 0;
        let mut this_pass_maxiter = 256;
        let mut maxiter_scale = 256;

        while self.iters > this_pass_maxiter {
            self.iters -= this_pass_maxiter;
            passcount += 1;
            if passcount & 1 == 1 {
                maxiter_scale = this_pass_maxiter / 2;
            }
            this_pass_maxiter += maxiter_scale;
        }
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use const_default::ConstDefault;

    use super::PointResult;

    #[test]
    fn cull_known_answers() {
        let cases = [
            (1, 1),
            (256, 256),
            (257, 1),
            (640, 384),
            (641, 1),
            (1152, 512),
            (1153, 1),
            (1920, 768),
            (1921, 1),
            (2944, 1024),
            (2945, 1),
            (4480, 1536),
            (4481, 1),
            (6528, 2048),
            (6529, 1),
            (9600, 3072),
            (9601, 1),
        ];

        let mut pr = PointResult::DEFAULT;
        for (input, expect) in cases {
            pr.iters = input;
            pr.cull_iterations();
            assert_eq!(pr.iters, expect, "input is {input}");
        }
    }
}
