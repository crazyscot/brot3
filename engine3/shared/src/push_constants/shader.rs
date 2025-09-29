use super::*;
#[cfg(not(target_arch = "spirv"))]
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "spirv"), derive(NoUninit))]
#[repr(C)]
pub struct FragmentConstants {
    pub size: Size,
}
