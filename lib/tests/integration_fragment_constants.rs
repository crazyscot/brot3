#![allow(missing_docs, clippy::float_cmp)]

use brot3_lib::{
    BigVec2,
    data::{Algorithm, Flags, FragmentConstants, Palette, PushExponent},
    ui::{UiState, ViewportZoom},
};
use glam::uvec2;

fn test_ui_state() -> UiState {
    UiState {
        viewport_translate: BigVec2::try_new(-0.75, 0.1).expect("failed to create BigVec2"),
        viewport_zoom: ViewportZoom::from(2.5_f32),
        algorithm: Algorithm::Mandelbrot,
        max_iter: 256,
        palette: Palette::default(),
        exponent: PushExponent::from(2),
        iteration_cull: false,
        viewport_size: uvec2(800, 600),
    }
}

#[test]
fn fragment_constants_from_uistate_preserves_algorithm() {
    let states = vec![
        {
            let mut s = test_ui_state();
            s.algorithm = Algorithm::Mandelbrot;
            s
        },
        {
            let mut s = test_ui_state();
            s.algorithm = Algorithm::BurningShip;
            s
        },
        {
            let mut s = test_ui_state();
            s.algorithm = Algorithm::Mandelbar;
            s
        },
    ];

    for state in states {
        let consts = FragmentConstants::from(&state);
        assert_eq!(consts.algorithm, state.algorithm);
    }
}

#[test]
fn fragment_constants_flags_iteration_cull() {
    let mut state = test_ui_state();
    state.iteration_cull = false;
    let consts = FragmentConstants::from(&state);
    assert!(!consts.flags.contains(Flags::ITERATION_CULL));

    state.iteration_cull = true;
    let consts = FragmentConstants::from(&state);
    assert!(consts.flags.contains(Flags::ITERATION_CULL));
}

#[test]
fn fragment_constants_max_iter_roundtrip() {
    let max_iters = vec![10, 50, 256, 1000, 5000, 10000];

    for max_iter in max_iters {
        let mut state = test_ui_state();
        state.max_iter = max_iter;
        let consts = FragmentConstants::from(&state);
        assert_eq!(consts.max_iter, max_iter);
    }
}

#[test]
fn fragment_constants_palette_preserved() {
    let mut state = test_ui_state();
    state.palette.gradient = 2.5;
    state.palette.saturation = 75.0;
    state.palette.lightness = 55.0;

    let consts = FragmentConstants::from(&state);

    assert_eq!(consts.palette.gradient, 2.5);
    assert_eq!(consts.palette.saturation, 75.0);
    assert_eq!(consts.palette.lightness, 55.0);
}

#[test]
fn fragment_constants_viewport_zoom_preserved() {
    #![allow(clippy::cast_possible_truncation)]
    let zoom_values = vec![0.5, 1.0, 2.5, 10.0, 100.0, 1e10];

    for zoom in zoom_values {
        let mut state = test_ui_state();
        state.viewport_zoom = ViewportZoom::from(zoom as f32);
        let consts = FragmentConstants::from(&state);

        // ViewportZoom has some internal representation; check it round-trips
        assert_eq!(
            ViewportZoom::from(consts.viewport_zoom),
            state.viewport_zoom
        );
    }
}
