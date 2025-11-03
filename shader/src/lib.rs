//! GPU shader implementing fractal rendering.
//! Can also be used on the host.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use spirv_std::glam::{f32, vec2, Vec2, Vec3, Vec4, Vec4Swizzles as _};
use spirv_std::spirv;

use shader_common::{
    FragmentConstants, PointResult, PointResultA, PointResultB, RenderStyle, GRID_SIZE,
};
use shader_util::grid::{GridRef, GridRefMut};

pub use shader_common::{Complex, INSPECTOR_MARKER_SIZE};

pub mod colour;
pub mod exponentiation;
pub mod fractal;

fn new_york_distance(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

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
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid_a: &mut [PointResultA],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] grid_b: &mut [PointResultB],
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
        let mut cache_a = GridRefMut::new(GRID_SIZE, grid_a);
        cache_a.set(coord.as_uvec2(), render_data.a());
        let mut cache_b = GridRefMut::new(GRID_SIZE, grid_b);
        cache_b.set(coord.as_uvec2(), render_data.b());
        render_data
    } else {
        let cache_a = GridRef::new(GRID_SIZE, grid_a);
        let a = cache_a.get(coord.as_uvec2());
        let cache_b = GridRef::new(GRID_SIZE, grid_b);
        let b = cache_b.get(coord.as_uvec2());
        PointResult::join(a, b)
    };

    let mut colour = colour::colour_data(render_data, constants);

    // Draw the inspector marker
    if constants.inspector_active.into() {
        // New York distance from the reference point draws a diamond shape
        let dist = new_york_distance(constants.inspector_point_pixel_address, coord);
        if dist < INSPECTOR_MARKER_SIZE * 0.667 {
            colour = Vec3::splat(0.0); // TODO Do something better here? Change pixels underneath?
        } else if dist < INSPECTOR_MARKER_SIZE {
            colour = Vec3::splat(1.0);
        }
    }

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
    use super::{FragmentConstants, PointResultA, PointResultB, GRID_SIZE};

    use shader_common::{Algorithm, Palette, PushExponent, RenderStyle};
    use shader_util::Size;
    use spirv_std::glam::{vec2, vec4, Vec2, Vec4};

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
            inspector_active: false.into(),
            inspector_point_pixel_address: Vec2::default(),
            render_style: RenderStyle::default(),
        }
    }

    #[test]
    fn fragment() {
        let mut res = Vec4::default();
        let consts = test_frag_consts();
        let mut grid_a = vec![PointResultA::default(); (GRID_SIZE.x * GRID_SIZE.y) as usize];
        let mut grid_b = vec![PointResultB::default(); (GRID_SIZE.x * GRID_SIZE.y) as usize];
        super::main_fs(
            vec4(0., 0., 0., 0.),
            &consts,
            &mut grid_a,
            &mut grid_b,
            &mut res,
        );
        let expected = vec4(0.49196184, 1., 0., 1.);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );
    }
}
