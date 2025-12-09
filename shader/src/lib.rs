//! GPU shader implementing fractal rendering.
//! Can also be used on the host.

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! ## Feature flags
#![doc = document_features::document_features!()]
#![allow(missing_docs)]

use spirv_std::glam::Vec4Swizzles as _;
pub use spirv_std::glam::{DVec2, UVec2, Vec2, Vec3, Vec4, f32, uvec2, vec2};
#[allow(unused_imports)] // Some are reused in some configurations
use spirv_std::spirv;

pub mod colour;
pub mod colourspace;
pub mod exponentiation;
pub mod fractal;
pub mod grid;

use colourspace::RgbVec;
use grid::{GridRef, GridRefMut, GridShared};
pub use shader_common::{Complex, INSPECTOR_MARKER_SIZE};
use shader_common::{Flags, FragmentConstants, data::PointResult};

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
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &mut [PointResult],
    output: &mut Vec4,
) {
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5); we
    // ignore depth & 1/w
    let coord = frag_coord.xy();
    let coord_int = coord.as_uvec2();

    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();

    // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
    let complex_coord = (coord - 0.5 * size) * pixel_spacing + constants.viewport_translate;

    let cache = GridRef::new(constants.buffer_size.as_uvec2(), grid);
    let cacheable = cache.address_valid(coord_int);

    let render_data = if !cacheable {
        fractal::render(constants, complex_coord)
    } else if constants.flags.contains(Flags::NEEDS_REITERATE) {
        let render_data = fractal::render(constants, complex_coord);
        let mut cache = GridRefMut::new(constants.buffer_size.as_uvec2(), grid);
        cache.set(coord_int, render_data);
        render_data
    } else {
        // it's cacheable and cached
        cache.get(coord_int)
    };

    let mut colour = colour::colour_data(render_data, constants, pixel_spacing);

    // Draw the inspector marker
    if constants.flags.contains(Flags::INSPECTOR_ACTIVE) {
        // New York distance from the reference point draws a diamond shape
        let dist = new_york_distance(constants.inspector_point_pixel_address, coord);
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
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    #[allow(clippy::cast_precision_loss)]
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    // uv expresses the cycle: (0,0) (2,0) (0,2) (2,2)
    let pos = 2.0 * uv - Vec2::ONE;
    // pos expresses the cycle: (-1,-1) (3,-1) (-1,3) (3,3)

    *out_pos = pos.extend(0.0).extend(1.0);
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::missing_panics_doc)]
mod tests {
    use const_default::ConstDefault as _;
    use float_eq::assert_float_eq;
    use shader_common::{Flags, Palette, PushExponent, Size, data::PointResult, enums::Algorithm};
    use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, uvec2, vec2, vec4};

    use super::{FragmentConstants, new_york_distance};

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
            super::main_vs(*id, &mut res);
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
            exponent: PushExponent::from(2),
            palette: Palette::DEFAULT,
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn render_save_retrieve() {
        #![allow(clippy::float_cmp)]

        use shader_common::Flags;
        let mut res = Vec4::default();
        let mut grid = vec![PointResult::default(); (TEST_GRID_SIZE.x * TEST_GRID_SIZE.y) as usize];

        let no_iterate = FragmentConstants {
            flags: Flags::empty(),
            ..test_frag_consts()
        };

        // Cache starts out empty (you probably couldn't run this on an actual GPU, the NaN might
        // trigger an abort)
        super::main_fs(vec4(0., 0., 0., 0.), &no_iterate, &mut grid, &mut res);
        assert!(res[0].is_nan());
        assert!(res[1].is_nan());
        assert!(res[2].is_nan());
        assert_eq!(res[3], 1.0);

        // Pass 1 populates cache
        super::main_fs(
            vec4(0., 0., 0., 0.),
            &test_frag_consts(),
            &mut grid,
            &mut res,
        );
        let expected = vec4(0.0, 1.0, 0.141_448_5, 1.0);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );

        // Pass 2: Retrieve from cache
        super::main_fs(vec4(0., 0., 0., 0.), &no_iterate, &mut grid, &mut res);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );
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
            ((9.0, 0.0), (0.0, 1.0, 0.141_448_5)),
        ];

        for (point, expect_rgb) in cases {
            let mut res = Vec4::default();
            let mut grid =
                vec![PointResult::default(); (TEST_GRID_SIZE.x * TEST_GRID_SIZE.y) as usize];

            // Set up to inspect the pixel we're rendering
            let inspector = FragmentConstants {
                flags: Flags::INSPECTOR_ACTIVE | Flags::NEEDS_REITERATE,
                inspector_point_pixel_address: Vec2::from(*point),
                ..test_frag_consts()
            };
            super::main_fs(vec4(0., 0., 0., 0.), &inspector, &mut grid, &mut res);
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
}
