//! UI saveable state
// (c) 2026 Ross Younger

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Error as LibError, ViewportZoom};
use crate::{
    BigVec2, UVec2, Vec2,
    data::{Algorithm, Flags, FragmentConstants, Palette, PushExponent},
    engine::NOMINAL_WINDOW_SIZE,
    util::Size,
};

#[derive(Clone, Debug, derive_more::PartialEq, Serialize, Deserialize)]
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
    /// The size of the viewport (window), in pixels.
    ///
    /// **This is a runtime parameter. It is not serialised, nor checked by [`PartialEq`].**
    #[serde(skip)]
    #[partial_eq(skip)]
    pub viewport_size: UVec2,
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
            viewport_size: NOMINAL_WINDOW_SIZE,
        }
    }
}

impl From<&UiState> for FragmentConstants {
    fn from(state: &UiState) -> Self {
        Self {
            flags: Flags::flag_if(state.iteration_cull, Flags::ITERATION_CULL),
            viewport_translate: state.viewport_translate.as_vec2(),
            viewport_zoom: state.viewport_zoom.into(),
            size: state.viewport_size.into(),
            buffer_size: Size::ZERO,
            algorithm: state.algorithm,
            max_iter: state.max_iter,
            exponent: state.exponent,
            palette: state.palette,
            inspector_point_pixel_address: Vec2::ZERO,
        }
    }
}

impl UiState {
    /// Replaces this struct with the other, except for the unserialised fields (which are
    /// preserved).
    pub fn merge(&mut self, other: Self) {
        let vp_size = self.viewport_size;
        *self = other;
        self.viewport_size = vp_size;
    }

    /// Converts the UI state to a string for display purposes. This is used in the metadata
    /// and default filename of saved images.
    #[cfg(not(spirv))]
    #[must_use]
    pub fn display_string(&self, separator: char) -> String {
        use crate::{engine::DEFAULT_FRACTAL_PLANE_SIZE, ui::Exponent};

        format!(
            "{alg}{sep}z{zoom}{sep}max{max_iter}{sep}exp{exp}{sep}{colourer:?}",
            alg = self.algorithm,
            zoom = self.viewport_zoom.display_string(
                DEFAULT_FRACTAL_PLANE_SIZE,
                NOMINAL_WINDOW_SIZE.y,
                self.viewport_size.y
            ),
            max_iter = self.max_iter,
            sep = separator,
            exp = Exponent::from(self.exponent).display_string(),
            colourer = self.palette.colourer,
        )
    }

    #[cfg(not(spirv))]
    /// Saves this state object to a JSON save file (via the `UiStateSaveFile` intermediate type,
    /// for a version check)
    pub fn save(&self, path: &Path) -> Result<(), LibError> {
        UiStateSaveFile::from(self.clone()).save(&path)
    }

    #[cfg(not(spirv))]
    /// Loads a state object from a JSON save file (via the `UiStateSaveFile` intermediate type,
    /// for a version check)
    pub fn load_json(path: &impl AsRef<Path>) -> Result<Self, LibError> {
        UiStateSaveFile::load(path)
    }

    #[cfg(not(spirv))]
    /// Loads a state object from a PNG file containing a `uistate` chunk, which is expected to
    /// contain a JSON serialization of a `UiState`.
    ///
    /// This allows us to embed the state in the image file itself, which is useful for sharing
    /// and for exploring near an interesting point.
    pub fn load_png(path: &impl AsRef<Path>) -> Result<Self, LibError> {
        use std::{fs::File, io::BufReader};

        let decoder = png::Decoder::new(BufReader::new(File::open(path.as_ref())?));
        let reader = decoder.read_info().map_err(|e| {
            if let png::DecodingError::Format(_) = e {
                // It's probably not a PNG at all
                LibError::UnrecognisedFormat
            } else {
                LibError::PngDecode(e)
            }
        })?;
        for chunk in &reader.info().uncompressed_latin1_text {
            if chunk.keyword == "uistate" {
                // Woo-hoo! It's for us!
                return serde_json::from_str(&chunk.text).map_err(Into::into);
            }
        }
        Err(LibError::PngHadNoStateData)
    }

    #[cfg(not(spirv))]
    /// Loads the state from the given file, of any supported type (currently JSON or PNG).
    ///
    /// *NOTE:* Caller is responsible for figuring out whether to enable perturbation mode or other
    /// flags based on the new state.
    pub fn load_magic(path: &Path) -> Result<UiState, LibError> {
        // Some errors are fatal (e.g. file not found), but if the file is there and it's just not
        // valid JSON, we want to try loading as a PNG before giving up.
        match UiState::load_json(&path) {
            Ok(state) => {
                log::info!("Loaded from {}", path.display());
                Ok(state)
            }
            Err(LibError::Json(j)) => {
                if j.is_syntax() {
                    // It's not valid JSON, so try PNG
                    Ok(UiState::load_png(&path)?)
                } else {
                    // I/O error, valid JSON that couldn't deserialize, premature EOF: all fatal
                    Err(LibError::Json(j))
                }
            }
            Err(e) => Err(e),
        }
    }
}
#[cfg(not(spirv))]
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
    type Error = LibError;

    fn try_from(value: UiStateSaveFile) -> Result<Self, Self::Error> {
        if value.version != 1 {
            return Err(LibError::UnsupportedVersion(value.version));
        }
        Ok(value.state)
    }
}

impl UiStateSaveFile {
    pub fn save(&self, path: &impl AsRef<Path>) -> Result<(), LibError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &impl AsRef<Path>) -> Result<UiState, LibError> {
        let data = std::fs::read_to_string(path.as_ref())?;
        let save_file: UiStateSaveFile = serde_json::from_str(&data)?;
        save_file.try_into()
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
            viewport_size: NOMINAL_WINDOW_SIZE,
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
