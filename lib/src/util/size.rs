//! GPU-friendly representation of a two-dimensional `u32` vector
//!
//! Based on earlier work by Abel <abel465@gmail.com>, see <https://github.com/abel465/mandelbrot>

#[cfg(not(spirv))]
use core::str::FromStr;

use bytemuck::NoUninit;

use crate::{UVec2, Vec2, uvec2, vec2};

/// GPU-friendly representation of a two-dimensional `u32` vector
#[derive(Copy, Clone, Debug, NoUninit, PartialEq, Eq)]
#[repr(C)]
#[allow(missing_docs)] // self-explanatory !
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

    /// Constructor
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Calculates the aspect ratio
    ///
    /// ```
    /// # use brot3_lib::util::Size;
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
    /// # use brot3_lib::util::Size;
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
    /// # use brot3_lib::util::Size;
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
    /// # use brot3_lib::util::Size;
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

impl From<Size> for UVec2 {
    /// ```
    /// # use brot3_lib::util::Size;
    /// # use glam::UVec2;
    /// let sz = Size::new(100, 200);
    /// let uv: UVec2 = sz.into();
    /// assert_eq!(uv.x, 100);
    /// assert_eq!(uv.y, 200);
    /// ```
    fn from(sz: Size) -> Self {
        uvec2(sz.width, sz.height)
    }
}

#[cfg(not(spirv))]
impl FromStr for Size {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("'{s}' is not in the format 'WIDTH,HEIGHT'"));
        }

        let width = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid width: {e}"))?;
        let height = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid height: {e}"))?;

        Ok(Self { width, height })
    }
}

#[cfg(not(spirv))]
impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.width, self.height)
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

    #[test]
    fn from_str() {
        let sz: Size = "1920,1080".parse().unwrap();
        assert_eq!(sz.width, 1920);
        assert_eq!(sz.height, 1080);

        let _ = "jk.kjl,ujk".parse::<Size>().unwrap_err();
        let _ = "123".parse::<Size>().unwrap_err();
        let _ = "123,456,789".parse::<Size>().unwrap_err();
    }
}
