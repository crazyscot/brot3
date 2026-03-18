//! SPIR-V entrypoints (testing)
//!
//! (c) 2025-6 Ross Younger, with inspiration from earlier work by Abel <abel465@gmail.com>; see <https://github.com/abel465/mandelbrot>

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use float_eq::assert_float_eq;
    use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, uvec2, vec2, vec4};

    use crate::{
        data::{Algorithm, Colourer, Flags, FragmentConstants, Palette, PointResult, PushExponent},
        engine::new_york_distance,
        util::Size,
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
            exponent: PushExponent::from(2),
            palette: Palette {
                colourer: Colourer::LogRainbow,
                ..Default::default()
            },
            inspector_point_pixel_address: Vec2::default(),
        }
    }

    #[test]
    fn render_save_retrieve() {
        #![allow(clippy::float_cmp)]

        let mut res = Vec4::default();
        let mut grid = vec![PointResult::default(); (TEST_GRID_SIZE.x * TEST_GRID_SIZE.y) as usize];

        let no_iterate = FragmentConstants {
            flags: Flags::empty(),
            ..test_frag_consts()
        };

        let mut empty = vec![]; // Complex

        // Cache starts out empty (you probably couldn't run this on an actual GPU, the NaN might
        // trigger an abort)
        crate::main_fs(
            vec4(0., 0., 0., 0.),
            &no_iterate,
            &mut grid,
            empty.as_mut(),
            &mut res,
        );
        assert!(res[0].is_nan());
        assert!(res[1].is_nan());
        assert!(res[2].is_nan());
        assert_eq!(res[3], 1.0);

        // Pass 1 populates cache
        crate::main_fs(
            vec4(0., 0., 0., 0.),
            &test_frag_consts(),
            &mut grid,
            empty.as_mut(),
            &mut res,
        );
        let expected = vec4(0.0, 1.0, 0.141_448_5, 1.0);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );

        // Pass 2: Retrieve from cache
        crate::main_fs(
            vec4(0., 0., 0., 0.),
            &no_iterate,
            &mut grid,
            empty.as_mut(),
            &mut res,
        );
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
        let mut empty = vec![]; // Complex

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
            crate::main_fs(
                vec4(0., 0., 0., 0.),
                &inspector,
                &mut grid,
                empty.as_mut(),
                &mut res,
            );
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
