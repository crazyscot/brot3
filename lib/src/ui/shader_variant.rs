//! Shader variant selection for runtime and build orchestration.

use super::{UiState, build_defs::ShaderVariant};
use crate::data::Algorithm;

impl ShaderVariant {
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
        assert_eq!(
            ShaderVariant::prebuild_shaders_dir_env_var(),
            "BROT3_PREBUILD_SHADERS_DIR"
        );
        assert_eq!(
            ShaderVariant::General.shader_filename(),
            "brot3_general.spv"
        );
        assert_eq!(
            ShaderVariant::MandelbrotPow2.shader_filename(),
            "brot3_mandelbrot_pow2.spv"
        );
        assert_eq!(
            ShaderVariant::MandelbrotPow2Deep.shader_filename(),
            "brot3_mandelbrot_pow2_deep.spv"
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
                "standard-mode",
            ]
        );
        assert_eq!(
            ShaderVariant::MandelbrotPow2.shader_crate_features(),
            &["all-colourers", "standard-mode"]
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
