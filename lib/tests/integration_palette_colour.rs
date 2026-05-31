#![allow(missing_docs, clippy::float_cmp)]

use brot3_lib::{
    BigVec2,
    data::{Algorithm, Colourer, FragmentConstants, Modifier, Palette, PushExponent},
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
fn palette_gradient_preserved_in_constants() {
    let gradient_values = vec![0.0, 1.0, 2.5, 5.0, 10.0];

    for gradient in gradient_values {
        let mut state = test_ui_state();
        state.palette.gradient = gradient;

        let consts = FragmentConstants::from(&state);

        assert_eq!(
            consts.palette.gradient, gradient,
            "Gradient {gradient} should be preserved",
        );
    }
}

#[test]
fn palette_saturation_lightness_preserved() {
    let mut state = test_ui_state();
    state.palette.saturation = 75.0;
    state.palette.lightness = 55.0;

    let consts = FragmentConstants::from(&state);

    assert_eq!(consts.palette.saturation, 75.0);
    assert_eq!(consts.palette.lightness, 55.0);
}

#[test]
fn palette_colourer_variants() {
    let colourers = vec![Colourer::BlackFade, Colourer::IcyBlue, Colourer::Neon];

    for colourer in colourers {
        let mut state = test_ui_state();
        state.palette.colourer = colourer;

        let consts = FragmentConstants::from(&state);

        assert_eq!(
            consts.palette.colourer, colourer,
            "Colourer {colourer:?} should be preserved",
        );
    }
}

#[test]
fn distance_estimate_flag_with_filaments_modifier() {
    use brot3_lib::data::Flags;

    let mut state = test_ui_state();
    // Default has no filaments modifier
    let consts = FragmentConstants::from(&state);
    assert!(!consts.flags.contains(Flags::DISTANCE_ESTIMATE));

    // Add filaments to brightness_style
    state.palette.brightness_style = Modifier::Filaments;
    let consts = FragmentConstants::from(&state);
    assert!(
        consts.flags.contains(Flags::DISTANCE_ESTIMATE),
        "Distance estimate should be enabled with Filaments brightness"
    );
}
