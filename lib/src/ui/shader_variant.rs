//! Shader variant selection for runtime and build orchestration.

use super::UiState;
use crate::data::Algorithm;

const GENERAL_FEATURES: &[&str] = &[
    "all-colourers",
    "all-fractals",
    "variable-exponent",
    "perturbation-mode",
];
const MANDELBROT_POW2_FEATURES: &[&str] = &["all-colourers"];
const MANDELBROT_POW2_DEEP_FEATURES: &[&str] = &["all-colourers", "perturbation-mode"];

/// The current planned shader variants.
///
/// These keys are intended to stay stable so they can be reused by build-time packaging and
/// runtime selection.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, strum::IntoStaticStr)]
pub enum ShaderVariant {
    /// The catch-all shader with every currently required capability enabled.
    #[default]
    #[strum(serialize = "general")]
    General,
    /// Mandelbrot-only shader for exponent 2 without perturbation mode.
    #[strum(serialize = "mandelbrot_pow2")]
    MandelbrotPow2,
    /// Mandelbrot-only shader for exponent 2 with perturbation mode enabled.
    #[strum(serialize = "mandelbrot_pow2_deep")]
    MandelbrotPow2Deep,
}

impl ShaderVariant {
    /// Stable identifier for this shader variant.
    #[must_use]
    pub fn key(self) -> &'static str {
        self.into()
    }

    /// Cargo feature flags needed to build this variant of `brot3-lib`.
    #[must_use]
    pub const fn shader_crate_features(self) -> &'static [&'static str] {
        match self {
            Self::General => GENERAL_FEATURES,
            Self::MandelbrotPow2 => MANDELBROT_POW2_FEATURES,
            Self::MandelbrotPow2Deep => MANDELBROT_POW2_DEEP_FEATURES,
        }
    }

    /// Chooses the preferred shader variant for a given UI state.
    #[must_use]
    pub fn from_ui_state(state: &UiState) -> Self {
        if state.algorithm != Algorithm::Mandelbrot || !state.exponent.is_two() {
            return Self::General;
        }
        if state.viewport_zoom.requires_perturbation_mode() {
            Self::MandelbrotPow2Deep
        } else {
            Self::MandelbrotPow2
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::ShaderVariant;
    use crate::{
        data::{Algorithm, PushExponent},
        ui::{UiState, ViewportZoom},
    };

    fn base_state() -> UiState {
        UiState {
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
            ..UiState::default()
        }
    }

    #[test]
    fn variant_keys_are_stable() {
        assert_eq!(ShaderVariant::General.key(), "general");
        assert_eq!(ShaderVariant::MandelbrotPow2.key(), "mandelbrot_pow2");
        assert_eq!(
            ShaderVariant::MandelbrotPow2Deep.key(),
            "mandelbrot_pow2_deep"
        );
    }

    #[test]
    fn feature_sets_match_the_planned_matrix() {
        assert_eq!(
            ShaderVariant::General.shader_crate_features(),
            &[
                "all-colourers",
                "all-fractals",
                "variable-exponent",
                "perturbation-mode",
            ]
        );
        assert_eq!(
            ShaderVariant::MandelbrotPow2.shader_crate_features(),
            &["all-colourers"]
        );
        assert_eq!(
            ShaderVariant::MandelbrotPow2Deep.shader_crate_features(),
            &["all-colourers", "perturbation-mode"]
        );
    }

    #[test]
    fn picks_general_for_non_mandelbrot_states() {
        let mut state = base_state();
        state.algorithm = Algorithm::BurningShip;
        assert_eq!(ShaderVariant::from_ui_state(&state), ShaderVariant::General);
    }

    #[test]
    fn picks_general_for_non_power_two_states() {
        let mut state = base_state();
        state.exponent = PushExponent::from(3);
        assert_eq!(ShaderVariant::from_ui_state(&state), ShaderVariant::General);
    }

    #[test]
    fn picks_specialized_shader_for_float_two() {
        let mut state = base_state();
        state.exponent = PushExponent::from(2.0_f32);
        assert_eq!(
            ShaderVariant::from_ui_state(&state),
            ShaderVariant::MandelbrotPow2
        );
    }

    #[test]
    fn picks_shallow_specialized_shader_for_standard_zoom() {
        let state = base_state();
        assert_eq!(
            ShaderVariant::from_ui_state(&state),
            ShaderVariant::MandelbrotPow2
        );
    }

    #[test]
    fn picks_deep_specialized_shader_when_zoom_requires_perturbation() {
        let mut state = base_state();
        state.viewport_zoom = ViewportZoom::from(ViewportZoom::MAX_ZOOM_STANDARD + 1.0);
        assert_eq!(
            ShaderVariant::from_ui_state(&state),
            ShaderVariant::MandelbrotPow2Deep
        );
    }
}
