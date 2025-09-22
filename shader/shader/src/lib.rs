#![no_std]

use spirv_std::glam::*;
//#[cfg(target_arch = "spirv")]
//use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

use shared::push_constants::shader::FragmentConstants;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    output: &mut Vec4,
) {
    let coord = frag_coord.xy();
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5)
    // we ignore depth & 1/w

    // map xy coords to (-0.5,0.5) in both dimensions whilst applying an aspect ratio fix
    let mut uv = (coord - 0.5 * vec2(constants.size.width as f32, constants.size.height as f32))
        / constants.size.height as f32;

    // both coords of uv are in range (-0.5,0.5); map this to colour space
    uv.x = 0.5 + uv.x;
    uv.y = 0.5 - uv.y; // invert y axis
    *output = vec4(0.0, uv.x, uv.y, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    // TODO: figure out what these inputs are and how to use them effectively - see spirv_std & SPIRV docs
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
