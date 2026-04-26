// (c) 2026 Ross Younger

#[cfg(not(spirv))]
use serde::{Deserialize, Serialize};

/// Newtype to centralise the display formatting logic
#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(spirv), derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct ViewportZoom(pub f64);

impl ViewportZoom {
    /// Maximum zoom with perturbation mode.
    /// Around this point, f32 maths breaks down: we can no longer accurately represent pixel sizes.
    // reported on UI as 1e35
    pub const MAX_ZOOM_PERTURBATIONS_F32: f64 = 2.5e34;
    /// The maximum zoom factor that can be achieved without perturbation mode.
    // reported on UI as 40000
    pub const MAX_ZOOM_STANDARD: f64 = 1.0e4;
    /// The minimum zoom factor we are interested in rendering.
    pub const MIN_ZOOM: f64 = 0.05;

    /// Relate the current axis size to the nominal initial size to get a more
    /// intuitive zoom readout.
    #[cfg(not(spirv))]
    #[must_use]
    pub fn display_string(
        self,
        fractal_plane_size: f64,
        default_y_axis_pixel_size: u32,
        y_axis_pixel_count: u32,
    ) -> String {
        use crate::engine::PixelSpacing as _;

        let initial_zoom = fractal_plane_size / f64::from(default_y_axis_pixel_size);
        // let zoom = initial_zoom / pixel_size;
        // = initial_zoom * pixel_size_inv
        let zoom = initial_zoom * self.0.pixel_spacing_inv(y_axis_pixel_count);

        if zoom < 1_000_000. {
            format!("{:.p$}", zoom, p = Self::zoom_precision(zoom))
        } else {
            format!("{zoom:.3e}")
        }
    }

    /// Precision digits for a zoom factor
    pub(crate) fn zoom_precision(v: f64) -> usize {
        match v {
            v if v < 10.0 => 3,
            v if v < 1000.0 => 2,
            v if v < 10000.0 => 1,
            _ => 0,
        }
    }

    /// Does this zoom factor require perturbation mode to render correctly?
    #[must_use]
    pub fn requires_perturbation_mode(self) -> bool {
        self.0 > Self::MAX_ZOOM_STANDARD
    }

    /// Clamp a zoom factor to the valid range for a given mode.
    #[must_use]
    pub fn clamp_to_mode(self, perturbation_mode: bool) -> Self {
        let max_zoom = if perturbation_mode {
            Self(Self::MAX_ZOOM_PERTURBATIONS_F32)
        } else {
            Self(Self::MAX_ZOOM_STANDARD)
        };
        Self(self.0.clamp(Self::MIN_ZOOM, max_zoom.0))
    }
}

impl From<ViewportZoom> for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: ViewportZoom) -> Self {
        value.0 as f32
    }
}
impl From<f32> for ViewportZoom {
    fn from(value: f32) -> Self {
        Self(value.into())
    }
}
impl From<f64> for ViewportZoom {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl PartialEq for ViewportZoom {
    fn eq(&self, other: &Self) -> bool {
        use crate::maths::FloatIsNear as _;
        self.0.is_near(other.0)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_display_string() {
        let zoom = ViewportZoom(0.25);
        assert_eq!(zoom.display_string(4.0, 600, 600), "1.000");
        let zoom = ViewportZoom(1.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "4.000");
        let zoom = ViewportZoom(10.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "40.00");
        let zoom = ViewportZoom(100.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "400.00");
        let zoom = ViewportZoom(1000.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "4000.0");
        let zoom = ViewportZoom(10_000.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "40000");
        let zoom = ViewportZoom(1_000_000.0);
        assert_eq!(zoom.display_string(4.0, 600, 600), "4.000e6");
    }

    #[test]
    fn equality_close() {
        let a = ViewportZoom(0.00025);
        let b = ViewportZoom(0.000_250_000_01);
        assert_eq!(a, b);
    }
}
