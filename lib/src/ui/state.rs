//! UI saveable state
// (c) 2026 Ross Younger

use serde::{Deserialize, Serialize};

use super::ViewportZoom;
use crate::{
    BigVec2,
    data::{Algorithm, FragmentConstants, Palette, PushExponent},
};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
/// The state of the UI, which can be saved and loaded
pub struct UiState {
    /// Viewport translation offset, in the complex plane. This is the point that the center of the
    /// viewport corresponds to.
    pub viewport_translate: BigVec2,
    /// Zoom factor for the viewport. 1.0 means the default view, 2.0 means zoomed in by a factor
    /// of 2, etc.
    pub viewport_zoom: ViewportZoom,
    /// Selects the fractal algorithm
    pub algorithm: Algorithm,
    /// The maximum number of iterations to compute
    pub max_iter: u32,
    /// The palette to use
    pub palette: Palette,
    /// The exponent for the fractal
    ///
    /// *N.B. This field is serialised as an `Exponent`!*
    pub exponent: PushExponent,
    /// Set to enable iteration cull mode. This affects the colouring.
    pub iteration_cull: bool,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            viewport_translate: BigVec2::try_new(-1., 0.)
                .unwrap()
                .with_precision(super::BIGNUM_PRECISION_LIMIT),
            viewport_zoom: FragmentConstants::DEFAULT_ZOOM.into(),
            algorithm: Algorithm::Mandelbrot,
            max_iter: FragmentConstants::DEFAULT_MAX_ITER,
            palette: Palette::default(),
            exponent: PushExponent::default(),
            iteration_cull: false,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStateSaveFile {
    /// The version of the save file format. This can be used to handle breaking changes in the
    /// future.
    pub version: u32,
    /// The actual UI state.
    pub state: UiState,
}

impl From<UiState> for UiStateSaveFile {
    fn from(state: UiState) -> Self {
        Self { version: 1, state }
    }
}

impl TryFrom<UiStateSaveFile> for UiState {
    type Error = anyhow::Error;

    fn try_from(value: UiStateSaveFile) -> anyhow::Result<Self> {
        anyhow::ensure!(
            value.version == 1,
            "Unsupported save file version {}",
            value.version
        );
        Ok(value.state)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod serde_tests {
    use serde_json;
    use strum::VariantArray;

    use super::*;
    use crate::data::Colourer;

    fn create_test_state() -> UiState {
        UiState {
            viewport_translate: BigVec2::try_new(1.0, 2.0).expect("failed to create BigVec2"),
            viewport_zoom: ViewportZoom::from(2.0_f32),
            algorithm: Algorithm::Mandelbrot,
            max_iter: 1000,
            palette: Palette::default(),
            exponent: PushExponent::from(2),
            iteration_cull: true,
        }
    }

    #[test]
    fn round_trip_basic() {
        let original = create_test_state();
        let json = serde_json::to_string_pretty(&original).expect("serialization failed");
        println!("Serialized JSON: {json}");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn viewport_zoom_values() {
        let mut state = create_test_state();

        // Test with large zoom (use f32 range)
        state.viewport_zoom = ViewportZoom::from(1e6_f32);
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state.viewport_zoom, deserialized.viewport_zoom);

        // Test with very small zoom (near MIN_POSITIVE for f32)
        state.viewport_zoom = ViewportZoom::from(f32::MIN_POSITIVE);
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state.viewport_zoom, deserialized.viewport_zoom);

        // Test with nominal zoom value 1.0
        state.viewport_zoom = ViewportZoom::from(1.0_f32);
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state.viewport_zoom, deserialized.viewport_zoom);
    }

    // ============================================================================
    // Complex Number Handling (BigVec2)
    // ============================================================================

    #[test]
    fn complex_zero() {
        let mut state = create_test_state();
        state.viewport_translate = BigVec2::try_new(0.0, 0.0).expect("failed to create BigVec2");

        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state, deserialized);
    }

    #[test]
    fn complex_unit_values() {
        let mut state = create_test_state();

        // 1 + 0i
        state.viewport_translate = BigVec2::try_new(1.0, 0.0).expect("failed to create BigVec2");
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state, deserialized);

        // 0 + 1i
        state.viewport_translate = BigVec2::try_new(0.0, 1.0).expect("failed to create BigVec2");
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state, deserialized);
    }

    #[test]
    fn complex_large_values() {
        let mut state = create_test_state();
        state.viewport_translate = BigVec2::try_new(1e15, -1e15).expect("failed to create BigVec2");

        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state, deserialized);
    }

    #[test]
    fn complex_negative_values() {
        let mut state = create_test_state();
        state.viewport_translate = BigVec2::try_new(-2.5, -3.7).expect("failed to create BigVec2");

        let json = serde_json::to_string_pretty(&state).expect("serialization failed");
        println!("Serialized JSON: {json}");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state, deserialized);
    }

    // ============================================================================
    // Enum Variants (Algorithm, Palette)
    // ============================================================================

    #[test]
    fn algorithm_variants() {
        for algo in Algorithm::VARIANTS {
            let mut state = create_test_state();
            state.algorithm = *algo;

            let json = serde_json::to_string(&state).expect("serialization failed");
            let deserialized: UiState =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(
                state, deserialized,
                "Algorithm variant not preserved: {algo:?}"
            );
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn palette_serialization() {
        // Test palette struct serialization with various field values
        let mut state = create_test_state();
        let mut palette = Palette::default();

        // Test with default values
        state.palette = palette;
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state.palette.colourer, deserialized.palette.colourer);
        assert_eq!(state.palette.gamma, deserialized.palette.gamma);

        // Test with modified palette values
        palette.gradient = 5.0;
        palette.offset = 2.5;
        palette.saturation = 75.0;
        state.palette = palette;
        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: UiState = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(state.palette.gradient, deserialized.palette.gradient);
        assert_eq!(state.palette.offset, deserialized.palette.offset);
        assert_eq!(state.palette.saturation, deserialized.palette.saturation);

        // Test all the colourer variants
        for colourer in Colourer::VARIANTS {
            state.palette = palette;

            let json = serde_json::to_string(&state).expect("serialization failed");
            let deserialized: UiState =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(
                state.palette.colourer, deserialized.palette.colourer,
                "Palette colourer variant not preserved: {colourer:?}"
            );
        }
    }

    // ============================================================================
    // Deserialization Error Handling
    // ============================================================================

    #[test]
    fn missing_required_fields() {
        let incomplete_json = r#"{"max_iter": 1000, "iteration_cull": true}"#;
        let err = serde_json::from_str::<UiState>(incomplete_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains("missing field"),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn type_mismatch_max_iter() {
        let invalid_json = r#"{
            "viewport_translate": {"x":["1",0],"y":["1",1]},
            "viewport_zoom": 2.0,
            "algorithm": "Mandelbrot",
            "max_iter": "not_a_number",
            "palette": {"colourer":"BlackFade","colour_style":"Continuous","brightness_style":"Standard","saturation_style":"Standard","gradient":1.0,"offset":0.0,"saturation":100.0,"lightness":50.0,"gamma":1.9},
            "exponent": {"integer":2},
            "iteration_cull": true
        }"#;
        let err = serde_json::from_str::<UiState>(invalid_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains("invalid type: string \"not_a_number\""),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn type_mismatch_iteration_cull() {
        let invalid_json = r#"{
            "viewport_translate": {"x":["1",0],"y":["1",1]},
            "viewport_zoom": 2.0,
            "algorithm": "Mandelbrot",
            "max_iter": 1000,
            "palette": {"colourer":"BlackFade","colour_style":"Continuous","brightness_style":"Standard","saturation_style":"Standard","gradient":1.0,"offset":0.0,"saturation":100.0,"lightness":50.0,"gamma":1.9},
            "exponent": {"integer":2},
            "iteration_cull": 1
        }"#;
        let err = serde_json::from_str::<UiState>(invalid_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains("invalid type: integer `1`"),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn unknown_enum_variant() {
        let mut state = create_test_state();
        state.max_iter = 500;

        let json = serde_json::to_string(&state).expect("serialization failed");
        let modded_json = json.replace(
            r#""algorithm":"Mandelbrot""#,
            r#""algorithm":"__UnknownAlgorithm__""#,
        );
        let err = serde_json::from_str::<UiState>(&modded_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains(r"unknown variant `__UnknownAlgorithm__`, expected one of"),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn invalid_palette_structure() {
        // Palette is a struct, not a string, so use invalid structure
        let invalid_json = r#"{
            "viewport_translate": {"x":["1",0],"y":["1",1]},
            "viewport_zoom": 2.0,
            "algorithm": "Mandelbrot",
            "max_iter": 1000,
            "palette": "not_a_palette_object",
            "exponent": {"integer":2},
            "iteration_cull": true
        }"#;
        let err = serde_json::from_str::<UiState>(invalid_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains(r#"invalid type: string "not_a_palette_object""#),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn malformed_json() {
        let malformed_json = r#"{
            "viewport_translate": {"x":["1",0],"y":["1",1]},
            "viewport_zoom": 2.0,
            "algorithm": "Mandelbrot",
            max_iter: 1000,
            "palette": "Twilight",
            "exponent": 2,
            "iteration_cull": true
        }"#;
        let err = serde_json::from_str::<UiState>(malformed_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains(r"key must be a string"),
            "Failure message not as expected: {err}"
        );
    }

    #[test]
    fn empty_json_object() {
        let empty_json = r"{}";
        let err = serde_json::from_str::<UiState>(empty_json)
            .expect_err("deserialization failed")
            .to_string();
        assert!(
            err.contains(r"missing field"),
            "Failure message not as expected: {err}"
        );
    }

    // ============================================================================
    // JSON file versioning
    // ============================================================================

    #[test]
    fn wrapper() {
        let original = create_test_state();
        let item = UiStateSaveFile::from(original.clone());
        let json = serde_json::to_string(&item).expect("serialization failed");
        println!("Serialized JSON: {json}");
        let deserialized: UiStateSaveFile =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(original, deserialized.try_into().unwrap());
    }
    #[test]
    fn convert_unknown_version() {
        let f = UiStateSaveFile {
            version: 999_999,
            state: UiState::default(),
        };
        let _ = UiState::try_from(f).expect_err("unknown version");
    }
}
