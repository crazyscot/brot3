// (c) 2026 Ross Younger

/// Newtype to centralise the display formatting logic
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ViewportZoom(pub f64);

impl ViewportZoom {
    /// Relate the current axis size to the nominal initial size to get a more
    /// intuitive zoom readout.
    #[cfg(not(target_arch = "spirv"))]
    #[must_use]
    pub fn display_string(
        self,
        fractal_plane_size: f64,
        default_y_axis_pixel_size: u32,
        y_axis_pixel_count: u32,
    ) -> String {
        use crate::PixelSpacing as _;

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

impl PartialEq for ViewportZoom {
    fn eq(&self, other: &Self) -> bool {
        use crate::FloatIsNear as _;
        self.0.is_near(other.0)
    }
}

#[cfg(all(test, not(target_arch = "spirv")))]
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
