//! GPU-friendly representation of a two-dimensional `u32` vector

use bytemuck::NoUninit;

use super::{UVec2, Vec2, uvec2, vec2};

/// GPU-friendly representation of a two-dimensional `u32` vector
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
    /// # use shader_common::Size;
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
    /// # use shader_common::Size;
    /// let sz = Size::new(100, 200);
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
    /// # use shader_common::Size;
    /// let sz = Size::new(100, 200);
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
    /// # use shader_common::Size;
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
    use float_eq::assert_float_eq;

    use super::Size;

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
