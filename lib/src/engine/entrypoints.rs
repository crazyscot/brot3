//! SPIR-V entrypoints (testing)
//!
//! (c) 2025-6 Ross Younger, with inspiration from earlier work by Abel <abel465@gmail.com>; see <https://github.com/abel465/mandelbrot>

use easy_cast::Cast as _;
#[allow(unused_imports)] // Some are reused in some configurations
use spirv_std::spirv;

use crate::{
    INSPECTOR_MARKER_SIZE, UVec3, Vec2, Vec3Swizzles as _, Vec4, Vec4Swizzles as _,
    data::{Flags, FragmentConstants},
    engine,
    util::{GridRefMut, PackedRgba8, RgbVec},
    vec2,
};

/// SPIRV `fragment` entrypoint.
/// This does the iteration and rendering work.
pub(crate) fn main_fs(
    frag_coord: Vec4,
    constants: &FragmentConstants,
    perturbation_reference_points: &[Vec2],
    output: &mut Vec4,
) {
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5); we
    // ignore depth & 1/w
    let coord = frag_coord.xy();

    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();

    // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;

    let render_data = engine::render(constants, complex_offset, perturbation_reference_points);

    let mut colour = engine::colour_data(render_data, constants);

    // Draw the inspector marker
    if constants.flags.contains(Flags::INSPECTOR_ACTIVE) {
        // New York distance from the reference point draws a diamond shape
        let dist = engine::new_york_distance(constants.inspector_point_pixel_address, coord);
        if dist < INSPECTOR_MARKER_SIZE * 0.667 {
            // TODO Do something better here? Change pixels underneath?
            colour = RgbVec::BLACK;
        } else if dist < INSPECTOR_MARKER_SIZE {
            colour = RgbVec::WHITE;
        }
    }

    *output = colour.0.extend(1.0);
}

/// SPIRV `vertex` entrypoint.
pub(crate) fn main_vs(vert_id: i32, out_pos: &mut Vec4) {
    let uv = vec2(((vert_id << 1) & 2).cast(), (vert_id & 2).cast());
    // uv expresses the cycle: (0,0) (2,0) (0,2) (2,2)
    let pos = 2.0 * uv - Vec2::ONE;
    // pos expresses the cycle: (-1,-1) (3,-1) (-1,3) (3,3)

    *out_pos = pos.extend(0.0).extend(1.0);
}

/// SPIRV compute shader entrypoint
pub(crate) fn main_cs(
    gid: UVec3,
    #[cfg(not(feature = "emulate_constants"))] constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")] constants: &FragmentConstants,
    pixels: &mut [u32],
    perturbation_reference_points: &[Vec2],
) {
    // Input gid is the pixel address
    let coord_int = gid.xy();
    let coord = coord_int.as_vec2();

    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();

    // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;

    let render_data = engine::render(constants, complex_offset, perturbation_reference_points);
    let colour = engine::colour_data(render_data, constants);

    // no inspector marker in compute shader

    let mut matrix = GridRefMut::new(constants.size.as_uvec2(), pixels);
    matrix.set(gid.xy(), PackedRgba8::from(colour).0);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use float_eq::assert_float_eq;

    use super::{PackedRgba8, RgbVec};
    use crate::{
        UVec2, Vec2, Vec3, Vec4,
        data::{Algorithm, Colourer, Flags, FragmentConstants, Palette, PushExponent},
        engine::new_york_distance,
        util::Size,
        uvec2, uvec3, vec2, vec3, vec4,
    };

    const TEST_GRID_SIZE: UVec2 = uvec2(2560, 1440);

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
            crate::main_vs(*id, &mut res);
            assert_eq!(&res, expected, "failing case: {id}");
        }
    }

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            flags: Flags::NEEDS_REITERATE,
            viewport_translate: vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(1024, 1024),
            buffer_size: TEST_GRID_SIZE.into(),
            max_iter: 10,
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::default(),
            palette: Palette {
                colourer: Colourer::Neon2,
                ..Default::default()
            },
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn an_inspector_calls() {
        let cases = &[
            // Centre of inspector is black, up to distance 6
            ((0.0, 0.0), (0.0, 0.0, 0.0)),
            ((5.0, 0.0), (0.0, 0.0, 0.0)),
            ((0.0, 5.0), (0.0, 0.0, 0.0)),
            ((5.0, 1.0), (0.0, 0.0, 0.0)),
            // 7 to 8 pixels out is white
            ((7.0, 0.0), (1.0, 1.0, 1.0)),
            ((8.0, 0.0), (1.0, 1.0, 1.0)),
            // 10 or more pixels out is unaltered
            ((9.0, 0.0), (0.0, 0.875, 0.875)),
        ];
        let mut empty = vec![]; // Complex

        for (point, expect_rgb) in cases {
            let mut res = Vec4::default();

            // Set up to inspect the pixel we're rendering
            let inspector = FragmentConstants {
                flags: Flags::INSPECTOR_ACTIVE | Flags::NEEDS_REITERATE,
                inspector_point_pixel_address: Vec2::from(*point),
                ..test_frag_consts()
            };
            crate::main_fs(vec4(0., 0., 0., 0.), &inspector, empty.as_mut(), &mut res);
            let expected = Vec3::from(*expect_rgb).extend(1.0);
            assert!(
                res.abs_diff_eq(expected, 0.000_000_1),
                "mismatch for point {point:?}: {res} vs {expected}"
            );
        }
    }

    #[test]
    fn big_apple() {
        let cases = &[
            ((0.0f32, 0.0f32), (0.0f32, 0.0f32), 0.0),
            ((0.0, 0.0), (0.0, 1.0), 1.0),
            ((0.0, 0.0), (1.0, 0.0), 1.0),
            ((0.0, 0.0), (1.0, 1.0), 2.0),
        ];
        for (a, b, result) in cases {
            assert_float_eq!(
                new_york_distance(Vec2::from(*a), Vec2::from(*b)),
                result,
                ulps <= 4
            );
        }
    }

    #[test]
    fn compute_shader() {
        let mut pixels = vec![0u32; (TEST_GRID_SIZE.x * TEST_GRID_SIZE.y) as usize];
        let perturbation_reference_points = vec![];

        let constants = test_frag_consts();

        crate::main_cs(
            uvec3(0, 0, 0),
            &constants,
            &mut pixels,
            &perturbation_reference_points,
        );

        let expected_colour = PackedRgba8::from(RgbVec::from(vec3(0.0, 0.8745, 0.8745))).0;
        assert_eq!(pixels[0], expected_colour);
    }
}
