//! Shader variant selection for runtime and build orchestration.

use super::UiState;
use crate::data::Algorithm;

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
    strum::FromRepr,
    strum::EnumIter,
)]
#[allow(missing_docs)]
/// Represents the different shader variants available for rendering
pub enum ShaderVariant {
    General,
    #[default]
    MandelbrotPow2,
    MandelbrotPow2Deep,
}

impl ShaderVariant {
    /// Chooses the preferred shader variant for a given UI state.
    #[must_use]
    pub fn from_ui_state(state: &UiState) -> Self {
        Self::from_ui_state_with_perturbation_mode(
            state,
            state.viewport_zoom.requires_perturbation_mode(),
        )
    }

    /// Chooses the preferred shader variant for a given UI state plus the active perturbation
    /// mode.
    #[must_use]
    pub fn from_ui_state_with_perturbation_mode(state: &UiState, perturbation_mode: bool) -> Self {
        if state.algorithm != Algorithm::Mandelbrot
            || !state.exponent.is_two()
            || *state.palette.brightness_style == crate::data::Modifier::Filaments
        {
            return Self::General;
        }
        if perturbation_mode {
            Self::MandelbrotPow2Deep
        } else {
            Self::MandelbrotPow2
        }
    }
}

impl From<&UiState> for ShaderVariant {
    fn from(state: &UiState) -> Self {
        Self::from_ui_state_with_perturbation_mode(
            state,
            state.viewport_zoom.requires_perturbation_mode(),
        )
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
            exponent: PushExponent::default(),
            ..UiState::default()
        }
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
        state.exponent = PushExponent::try_from(3).unwrap();
        assert_eq!(ShaderVariant::from_ui_state(&state), ShaderVariant::General);
    }

    #[test]
    fn picks_specialized_shader_for_float_two() {
        let mut state = base_state();
        state.exponent = PushExponent::try_from(2.0_f32).unwrap();
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

    #[test]
    fn picks_general_for_distance_estimate() {
        let mut state = base_state();
        *state.palette.brightness_style = crate::data::Modifier::Filaments;
        assert_eq!(ShaderVariant::from_ui_state(&state), ShaderVariant::General);
    }

    #[test]
    fn picks_deep_specialized_shader_when_perturbation_is_forced() {
        let state = base_state();
        assert_eq!(
            ShaderVariant::from_ui_state_with_perturbation_mode(&state, true),
            ShaderVariant::MandelbrotPow2Deep
        );
    }

    #[test]
    fn forced_perturbation_still_falls_back_to_general_for_unsupported_states() {
        let mut state = base_state();
        state.algorithm = Algorithm::BurningShip;
        assert_eq!(
            ShaderVariant::from_ui_state_with_perturbation_mode(&state, true),
            ShaderVariant::General
        );
    }
}
