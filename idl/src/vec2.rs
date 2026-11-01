//! Local wrapper that looks like `spirv_std::glam::Vec2`, but implements `WgslCompatible`

use bytemuck::NoUninit;
use glam::Vec2 as GlamVec2;
use my_wgsl::{WgslCompatible, WgslKind};

/// Local wrapper around `glam::Vec2`

#[derive(Copy, Clone, Debug, Default, NoUninit, PartialEq)]
#[repr(C)]
#[allow(missing_docs)]
pub struct WgslVec2 {
    pub x: f32,
    pub y: f32,
}

// Manually implement WgslCompatible to force the desired layout
impl WgslCompatible for WgslVec2 {
    const WGSL_ALIGN: usize = 8;
    const WGSL_ARRAY_LENGTH: usize = 0;
    const WGSL_DEFINE: &'static str = "";
    const WGSL_IDENT: &'static str = "vec2<f32>";
    const WGSL_KIND: WgslKind = WgslKind::Primitive;
    const WGSL_SIZE: usize = 8;
}

impl From<GlamVec2> for WgslVec2 {
    fn from(value: GlamVec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<WgslVec2> for GlamVec2 {
    fn from(value: WgslVec2) -> Self {
        Self::new(value.x, value.y)
    }
}

impl std::ops::Add<WgslVec2> for GlamVec2 {
    type Output = GlamVec2;

    fn add(self, rhs: WgslVec2) -> Self::Output {
        self + GlamVec2::from(rhs)
    }
}

impl std::ops::Sub<WgslVec2> for GlamVec2 {
    type Output = GlamVec2;

    fn sub(self, rhs: WgslVec2) -> Self::Output {
        self - GlamVec2::from(rhs)
    }
}
