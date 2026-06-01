#![allow(missing_docs)]

use std::fs;

use brot3_lib::{
    BigVec2,
    data::{Algorithm, Palette, PushExponent},
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
fn save_load_json_roundtrip_simple() {
    let original = UiState::default();
    let temp_file = std::env::temp_dir().join("test_roundtrip_simple.json");

    // Save and load
    original.save(&temp_file).expect("save failed");
    let loaded = UiState::load_json(&temp_file).expect("load failed");

    // Verify exact match
    assert_eq!(original, loaded);

    let _ = fs::remove_file(temp_file);
}

#[test]
fn save_load_json_roundtrip_complex() {
    let mut original = test_ui_state();
    original.viewport_zoom = ViewportZoom::from(1e10_f32);
    original.max_iter = 10000;
    original.palette.gradient = 5.0;

    let temp_file = std::env::temp_dir().join("test_roundtrip_complex.json");

    original.save(&temp_file).expect("save failed");
    let loaded = UiState::load_json(&temp_file).expect("load failed");

    assert_eq!(original, loaded);

    let _ = fs::remove_file(temp_file);
}

#[test]
fn state_from_constants_to_constants_roundtrip() {
    use brot3_lib::data::FragmentConstants;

    let original_state = test_ui_state();
    let consts = FragmentConstants::from(&original_state);
    let roundtrip_state = UiState::try_from(&consts).expect("conversion failed");

    // Fields that should survive the conversion exactly:
    assert_eq!(original_state.algorithm, roundtrip_state.algorithm);
    assert_eq!(original_state.max_iter, roundtrip_state.max_iter);
    assert_eq!(
        original_state.palette.colourer,
        roundtrip_state.palette.colourer
    );
    assert_eq!(original_state.exponent, roundtrip_state.exponent);
    assert_eq!(
        original_state.iteration_cull,
        roundtrip_state.iteration_cull
    );

    // Zoom should survive (ViewportZoom handles this)
    assert_eq!(original_state.viewport_zoom, roundtrip_state.viewport_zoom);

    // Note: viewport_translate loses precision in conversion due to f32 representation in
    // FragmentConstants and viewport_size is not preserved in the conversion
}

#[test]
fn save_load_with_algorithm_variants() {
    use strum::VariantArray;

    for algo in Algorithm::VARIANTS {
        let mut state = test_ui_state();
        state.algorithm = *algo;

        let temp_file = std::env::temp_dir().join(format!("test_algo_{algo:?}.json"));

        state.save(&temp_file).expect("save failed");
        let loaded = UiState::load_json(&temp_file).expect("load failed");

        assert_eq!(state, loaded, "Algorithm {algo:?} roundtrip failed");

        let _ = fs::remove_file(temp_file);
    }
}

#[test]
fn state_persistence_across_zoom_levels() {
    let zoom_levels = vec![0.1, 1.0, 10.0, 100.0, 1000.0, 1e10];

    for zoom in zoom_levels {
        let mut state = test_ui_state();
        state.viewport_zoom = ViewportZoom::from(zoom);

        let temp_file = std::env::temp_dir().join(format!("test_zoom_{zoom}.json"));

        state.save(&temp_file).expect("save failed");
        let loaded = UiState::load_json(&temp_file).expect("load failed");

        assert_eq!(
            state.viewport_zoom, loaded.viewport_zoom,
            "Zoom {zoom} mismatch",
        );

        let _ = fs::remove_file(temp_file);
    }
}
