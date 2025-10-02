#[cfg(not(target_arch = "spirv"))]
use bytemuck::NoUninit;

use shader_util::{Bool, Size};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct FragmentConstants {
    pub size: Size,
    pub needs_reiterate: Bool,
}

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct RenderData {
    pub iters: u32,
    pub smooth_iters: f32,
}

impl RenderData {
    pub fn new(_constants: &FragmentConstants, inside: bool, iters: u32) -> Self {
        let iters = if inside { u32::MAX } else { iters };
        Self {
            iters,
            smooth_iters: iters as f32,
        }
    }
}
