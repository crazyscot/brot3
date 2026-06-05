//! Build definitions control, shared between build scripts and runtime.
//!
//! Anywhere that uses this must provide strum with the `VariantArray` derive macro.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::VariantArray)]
pub enum ShaderVariant {
    General,
    MandelbrotPow2,
    MandelbrotPow2Deep,
}

const GENERAL_FEATURES: &[&str] = &[
    "all-fractals",
    "variable-exponent",
    "perturbation-mode",
    "standard-mode",
    "distance-estimate",
];
const MANDELBROT_POW2_FEATURES: &[&str] = &["standard-mode"];
const MANDELBROT_POW2_DEEP_FEATURES: &[&str] = &["perturbation-mode"];

impl ShaderVariant {
    #[must_use]
    pub const fn prebuild_shaders_dir_env_var() -> &'static str {
        "BROT3_PREBUILD_SHADERS_DIR"
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::MandelbrotPow2 => "mandelbrot_pow2",
            Self::MandelbrotPow2Deep => "mandelbrot_pow2_deep",
        }
    }

    #[must_use]
    pub const fn shader_filename(self) -> &'static str {
        match self {
            Self::General => "brot3_general.spv",
            Self::MandelbrotPow2 => "brot3_mandelbrot_pow2.spv",
            Self::MandelbrotPow2Deep => "brot3_mandelbrot_pow2_deep.spv",
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn shader_crate_features(self) -> &'static [&'static str] {
        match self {
            Self::General => GENERAL_FEATURES,
            Self::MandelbrotPow2 => MANDELBROT_POW2_FEATURES,
            Self::MandelbrotPow2Deep => MANDELBROT_POW2_DEEP_FEATURES,
        }
    }
}
