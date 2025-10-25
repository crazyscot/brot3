//! GPU shader implementing fractal rendering.
//! Can also be used on the host.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use spirv_std::glam::*;
use spirv_std::spirv;

use shader_common::{complex::Complex, FragmentConstants, PointResult, GRID_SIZE};
use shader_util::grid::{GridRef, GridRefMut};

pub mod colour;
pub mod exponentiation;
pub mod fractal;

/// SPIRV `fragment` entrypoint.
/// This does the iteration and rendering work.
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &mut [PointResult],
    output: &mut Vec4,
) {
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5); we ignore depth & 1/w
    let coord = frag_coord.xy();
    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();

    let render_data = if constants.needs_reiterate.into() {
        // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
        let cplx = (coord - 0.5 * size) / size.y / constants.viewport_zoom;
        let render_data = fractal::render(constants, cplx + constants.viewport_translate);
        let mut cache = GridRefMut::new(GRID_SIZE, grid);
        cache.set(coord.as_uvec2(), render_data);
        render_data
    } else {
        let cache = GridRef::new(GRID_SIZE, grid);
        cache.get(coord.as_uvec2())
    };

    let colour = colour::colour_data(render_data, constants);
    *output = colour.extend(1.0);
}

/// SPIRV `vertex` entrypoint.
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    // uv expresses the cycle: (0,0) (2,0) (0,2) (2,2)
    let pos = 2.0 * uv - Vec2::ONE;
    // pos expresses the cycle: (-1,-1) (3,-1) (-1,3) (3,3)

    *out_pos = pos.extend(0.0).extend(1.0);
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{FragmentConstants, PointResult, GRID_SIZE};

    use shader_common::{Algorithm, Palette, PushExponent};
    use shader_util::Size;
    use spirv_std::glam::{vec2, vec4, Vec4};

    #[test]
    fn vertex() {
        let cases = &[
            (0, vec4(-1., -1., 0., 1.)),
            (1, vec4(3., -1., 0., 1.)),
            (2, vec4(-1., 3., 0., 1.)),
            (3, vec4(3., 3., 0., 1.)),
            (4, vec4(-1., -1., 0., 1.)),
        ];

        for (id, expected) in cases {
            let mut res = Vec4::default();
            super::main_vs(*id, &mut res);
            assert_eq!(&res, expected, "failing case: {id}");
        }
    }

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            viewport_translate: vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(1024, 1024),
            max_iter: 10,
            needs_reiterate: true.into(),
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
            palette: Palette::default(),
            fractional_iters: true.into(),
        }
    }

    #[test]
    fn fragment() {
        let mut res = Vec4::default();
        let consts = test_frag_consts();
        let mut grid = vec![PointResult::default(); (GRID_SIZE.x * GRID_SIZE.y) as usize];
        super::main_fs(vec4(0., 0., 0., 0.), &consts, &mut grid, &mut res);
        let expected = vec4(0.5333053, 1., 0., 1.);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );
    }
}
