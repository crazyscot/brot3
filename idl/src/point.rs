//! Fractal data representation
//!
//! (c) 2025-6 Ross Younger

use bytemuck::NoUninit;
use my_wgsl::WgslCompatible;
use num_traits::AsPrimitive;

use super::{BoundaryClass, ColourStyle};
use crate::WgslEnum;

/// Raw data from a fractal invocation
#[derive(Copy, Clone, Debug, Default, NoUninit, WgslCompatible)]
#[repr(C)]
pub struct PointResult {
    /// iteration count; `u32::MAX` means that the limit was reached, signifying "inside the set"
    iters: u32,
    /// fractional part of iteration count (range 0..1)
    iters_fraction: f32,
    /// final angle (argument) (range -pi..pi)
    pub angle: f32,
    /// final complex distance, squared
    pub radius_sqr: f32,
    /// Is this point considered to be on the boundary?
    pub boundary: WgslEnum<BoundaryClass>,
}

impl const_default::ConstDefault for PointResult {
    const DEFAULT: Self = Self {
        iters: u32::MAX,
        iters_fraction: 0.0,
        angle: 0.0,
        radius_sqr: 0.0,
        boundary: WgslEnum(BoundaryClass::Indeterminate),
    };
}

impl PointResult {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new(
        iters: u32,
        iters_fraction: f32,
        angle: f32,
        radius_sqr: f32,
        boundary: BoundaryClass,
    ) -> Self {
        Self {
            iters,
            iters_fraction,
            angle,
            radius_sqr,
            boundary: WgslEnum(boundary),
        }
    }

    // ACCESSORS ////////////////////////////////////////////////////////////
    /// Iterations
    #[must_use]
    pub fn iters(&self, style: ColourStyle) -> f32 {
        // Cast with precision loss is OK here. The actual number is limited to MAX_MAX_ITER
        // (100,000), which is exactly representable in f32. The only issue that crops up is
        // our current use of `u32::MAX` to represent "infinity" for points inside the set.
        let whole = self.iters_whole().as_();
        let fractional = whole + self.iters_fraction();
        match style {
            ColourStyle::Discrete => whole,
            ColourStyle::Continuous => fractional,
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
    #[cfg(not(spirv))]
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
    ///
    /// INPUT CONDITION: `self.inside() == false` (caller is expected to have checked this)
    pub fn cull_iterations(&mut self) {
        // k := floor(log2(floor(iters / 640) + 1))
        let pair_index = self.iters / 640 + 1;
        let k = pair_index.ilog2();
        // r := iters - 640 * (2^k - 1)
        let scale = 1 << k;
        let r = self.iters - 640 * (scale - 1);
        let first_pass = 256 * scale;
        let result = if r < first_pass { r } else { r - first_pass };
        self.iters = result;
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use const_default::ConstDefault;

    use super::PointResult;

    #[test]
    fn cull_known_answers() {
        let cases = [
            (1, 1),
            (255, 255),
            (256, 0),
            (257, 1),
            (639, 383),
            (640, 0),
            (641, 1),
            (1151, 511),
            (1152, 0),
            (1919, 767),
            (1920, 0),
            (1921, 1),
            (2943, 1023),
            (2944, 0),
            (2945, 1),
            (4479, 1535),
            (4480, 0),
            (4481, 1),
            (6527, 2047),
            (6528, 0),
            (6529, 1),
            (9599, 3071),
            (9600, 0),
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
