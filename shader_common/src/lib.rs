#![cfg_attr(target_arch = "spirv", no_std)]
pub use abels_complex as complex;

use glam::{UVec2, Vec2, uvec2};

pub const GRID_SIZE: UVec2 = uvec2(3840, 2160);

#[cfg(not(target_arch = "spirv"))]
use bytemuck::NoUninit;

use shader_util::{Bool, Size};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct FragmentConstants {
    pub viewport_translate: Vec2,
    pub viewport_zoom: f32,
    /// window pixel size
    pub size: Size,
    pub algorithm: Algorithm,
    pub max_iter: u32,
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
    pub fn new(
        _constants: &FragmentConstants,
        inside: bool,
        iters: u32,
        smooth_iters: f32,
    ) -> Self {
        if inside {
            Self {
                iters: u32::MAX,
                smooth_iters: u32::MAX as f32,
            }
        } else {
            Self {
                iters,
                smooth_iters,
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(NoUninit, strum::EnumIter, strum::IntoStaticStr)
)]
#[repr(u32)]
pub enum Algorithm {
    #[default]
    Mandelbrot,
    Mandeldrop,
    // TODO others.. but impl the alg switch & reiterate first
}
