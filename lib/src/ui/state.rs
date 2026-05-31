//! UI saveable state
// (c) 2026 Ross Younger

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Error as LibError, ViewportZoom};
use crate::{
    BigVec2, UVec2, Vec2,
    data::{Algorithm, Flags, FragmentConstants, Modifier, Palette, PushExponent},
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
    /// *N.B. This field is serialised as [`crate::ui::Exponent`] !*
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
        let dist_est_required = state.palette.brightness_style == Modifier::Filaments;
        Self {
            // caution; this flags value is overridden by Controller::fragment_constants() if we've
            // come that way
            flags: Flags::flag_if(state.iteration_cull, Flags::ITERATION_CULL)
                | Flags::flag_if(dist_est_required, Flags::DISTANCE_ESTIMATE),
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

impl TryFrom<&FragmentConstants> for UiState {
    type Error = LibError;

    fn try_from(consts: &FragmentConstants) -> Result<Self, LibError> {
        Ok(Self {
            viewport_translate: BigVec2::try_new(
                consts.viewport_translate.x,
                consts.viewport_translate.y,
            )
            .map_err(|e| LibError::StateConversionFailed(e.to_string()))?
            .with_precision(super::BIGNUM_PRECISION_LIMIT),
            viewport_zoom: consts.viewport_zoom.into(),
            algorithm: consts.algorithm,
            max_iter: consts.max_iter,
            palette: consts.palette,
            exponent: consts.exponent,
            iteration_cull: consts.flags.contains(Flags::ITERATION_CULL),
            viewport_size: consts.size.into(),
        })
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
            Err(LibError::Io(e)) if e.kind() == std::io::ErrorKind::InvalidData => {
                // This is what we get when the file isn't valid UTF-8. Try PNG.
                Ok(UiState::load_png(&path)?)
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
            "palette": {"colourer":"BlackFade","colour_style":"Continuous","brightness_style":"Standard","gradient":1.0,"offset":0.0,"saturation":100.0,"lightness":50.0,"gamma":1.9},
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
            "palette": {"colourer":"BlackFade","colour_style":"Continuous","brightness_style":"Standard","gradient":1.0,"offset":0.0,"saturation":100.0,"lightness":50.0,"gamma":1.9},
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod conversion_tests {
    use glam::uvec2;

    use super::*;
    use crate::data::Modifier;

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

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            flags: Flags::empty(),
            viewport_translate: crate::vec2(-0.75, 0.1),
            viewport_zoom: 2.5,
            size: Size::new(800, 600),
            buffer_size: Size::ZERO,
            algorithm: Algorithm::Mandelbrot,
            max_iter: 256,
            exponent: PushExponent::from(2),
            palette: Palette::default(),
            inspector_point_pixel_address: Vec2::ZERO,
        }
    }

    #[test]
    fn uistate_from_fragment_constants() {
        let consts = test_frag_consts();
        let state = UiState::try_from(&consts).expect("conversion failed");

        assert_eq!(state.viewport_zoom, consts.viewport_zoom.into());
        assert_eq!(state.algorithm, consts.algorithm);
        assert_eq!(state.max_iter, consts.max_iter);
        assert_eq!(state.palette, consts.palette);
        assert_eq!(state.exponent, consts.exponent);
        assert!(!state.iteration_cull);
        assert_eq!(state.viewport_size, consts.size.into());
    }

    #[test]
    fn uistate_from_fragment_constants_with_iteration_cull() {
        let mut consts = test_frag_consts();
        consts.flags |= Flags::ITERATION_CULL;
        let state = UiState::try_from(&consts).expect("conversion failed");

        assert!(state.iteration_cull);
    }

    #[test]
    fn fragment_constants_from_uistate() {
        #![allow(clippy::float_cmp, clippy::cast_possible_truncation)]
        let state = test_ui_state();
        let consts = FragmentConstants::from(&state);

        assert_eq!(consts.viewport_zoom, state.viewport_zoom.0 as f32);
        assert_eq!(consts.algorithm, state.algorithm);
        assert_eq!(consts.max_iter, state.max_iter);
        assert_eq!(consts.palette, state.palette);
        assert_eq!(consts.exponent, state.exponent);
        assert!(!consts.flags.contains(Flags::ITERATION_CULL));
    }

    #[test]
    fn fragment_constants_from_uistate_with_flags() {
        let mut state = test_ui_state();
        state.iteration_cull = true;
        let consts = FragmentConstants::from(&state);

        assert!(consts.flags.contains(Flags::ITERATION_CULL));
    }

    #[test]
    fn fragment_constants_from_uistate_with_distance_estimate_flag() {
        let mut state = test_ui_state();
        let palette = Palette {
            brightness_style: Modifier::Filaments,
            ..Palette::default()
        };
        state.palette = palette;
        let consts = FragmentConstants::from(&state);

        assert!(consts.flags.contains(Flags::DISTANCE_ESTIMATE));
    }

    #[test]
    fn uistate_merge_preserves_viewport_size() {
        let mut state1 = test_ui_state();
        state1.viewport_size = uvec2(1024, 768);

        let mut state2 = test_ui_state();
        state2.viewport_size = uvec2(640, 480);
        state2.max_iter = 512;
        state2.algorithm = Algorithm::BurningShip;

        state1.merge(state2);

        assert_eq!(
            state1.viewport_size,
            uvec2(1024, 768),
            "merge should preserve viewport_size"
        );
        assert_eq!(state1.max_iter, 512);
        assert_eq!(state1.algorithm, Algorithm::BurningShip);
    }

    #[test]
    fn uistate_merge_overwrites_other_fields() {
        let mut state1 = test_ui_state();
        state1.max_iter = 256;

        let mut state2 = test_ui_state();
        state2.max_iter = 1024;

        state1.merge(state2);

        assert_eq!(state1.max_iter, 1024);
    }

    #[test]
    fn uistate_display_string() {
        let state = test_ui_state();
        let display = state.display_string('-');

        // display_string should be non-empty and contain algorithm name, max_iter, exponent
        assert!(!display.is_empty());
        assert!(display.contains("Mandelbrot"));
        assert!(display.contains("256")); // max_iter
        assert!(display.contains("exp2")); // exponent
    }

    #[test]
    fn uistate_display_string_with_different_separator() {
        let state = test_ui_state();
        let display_dash = state.display_string('-');
        let display_space = state.display_string(' ');

        // Both should have content but with different separators
        assert!(!display_dash.is_empty());
        assert!(!display_space.is_empty());
        assert!(display_dash.contains('-'));
        assert!(display_space.contains(' '));
    }

    #[test]
    fn uistate_save_and_load_json() {
        use std::fs;

        let state = test_ui_state();
        let temp_file = std::env::temp_dir().join("test_state.json");

        // Save
        state.save(&temp_file).expect("save failed");
        assert!(temp_file.exists());

        // Load
        let loaded = UiState::load_json(&temp_file).expect("load failed");

        assert_eq!(state, loaded);

        // Cleanup
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn uistate_load_magic_with_json() {
        use std::fs;

        let state = test_ui_state();
        let temp_file = std::env::temp_dir().join("test_magic.json");

        state.save(&temp_file).expect("save failed");

        let loaded = UiState::load_magic(&temp_file).expect("load_magic failed");

        assert_eq!(state, loaded);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn uistate_save_file_versioning() {
        let state = test_ui_state();
        let save_file = UiStateSaveFile::from(state.clone());

        assert_eq!(save_file.version, 1);
        assert_eq!(save_file.state, state);
    }

    #[test]
    fn uistate_save_file_unsupported_version() {
        let bad_save_file = UiStateSaveFile {
            version: 999,
            state: test_ui_state(),
        };

        let result: Result<UiState, _> = bad_save_file.try_into();
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), LibError::UnsupportedVersion(999)),
            "should be UnsupportedVersion(999)"
        );
    }
}
