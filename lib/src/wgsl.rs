//! Runtime WGSL assembly for the UI path.
//!
//! The shader is intentionally assembled from small, reusable fragments so the runtime path can
//! switch between a small set of variant kernels without maintaining a monolithic source string.

use brot3_idl::Colourer;
use log::debug;

use crate::ui::{ShaderKey, ShaderVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[allow(missing_docs)]
pub enum ShaderType {
    Compute,
    Fragment,
}

const COMMON: &str = include_str!("wgsl/common.wgsl");
const MAIN_FS: &str = include_str!("wgsl/main_fs.wgsl");
const MAIN_CS: &str = include_str!("wgsl/main_cs.wgsl");
const COLOUR_SHARED: &str = include_str!("wgsl/colour.wgsl");

// Variants may become templated at some point with mods for different fractal types
const VARIANT_GENERAL: &str = include_str!("wgsl/variant_general.wgsl");
const VARIANT_MANDELBROT_POW2: &str = include_str!("wgsl/variant_general.wgsl"); // TODO: becomes mandelbrot_pow2.wgsl
const VARIANT_MANDELBROT_POW2_DEEP: &str = include_str!("wgsl/variant_general.wgsl"); // TODO: becomes mandelbrot_pow2_deep.wgsl

#[must_use]
fn assemble_shader(typ: ShaderType, variant: ShaderVariant, colourer: Colourer) -> String {
    debug!("Assembling shader: type={typ:?}, variant={variant:?}, colourer={colourer:?}");
    let type_fragment = match typ {
        ShaderType::Compute => MAIN_CS,
        ShaderType::Fragment => MAIN_FS,
    };
    let variant_fragment = match variant {
        ShaderVariant::General => VARIANT_GENERAL,
        ShaderVariant::MandelbrotPow2 => VARIANT_MANDELBROT_POW2,
        ShaderVariant::MandelbrotPow2Deep => VARIANT_MANDELBROT_POW2_DEEP,
    };
    let colour_fragment = format!(
        r"
/* {colourer} */
fn colour_impl(data: PointResult, iters: f32) -> vec3<f32> {{
    return {};
}}
    ",
        colour_impl(colourer)
    );

    [
        "\n",
        &brot3_idl::ALL_WGSL,
        "\n",
        COMMON,
        "\n",
        type_fragment,
        "\n",
        COLOUR_SHARED,
        "\n",
        &colour_fragment,
        "\n",
        variant_fragment,
        "\n",
    ]
    .concat()
}

fn colour_impl(colourer: Colourer) -> &'static str {
    match colourer {
        Colourer::BlackFade => {
            "simple_cos(log( max(iters-3.0, 1.0) ), vec3<f32>(1.0, 2.0, 3.0), vec3<f32>(0.0), -0.5)"
        }
        Colourer::IcyBlue => {
            "powered_cos( log((iters + PI) / PI), vec3(PI, PI, 0.0), vec3(1.0, 1.0, -1.0), vec3(0.0, 0.0, 1.0), 4.0, 1.9 )"
        }
        Colourer::Mandy => {
            "
            simple_cos(
                sqrt( max(iters-3.0, 0.0) ) * TAU,
                vec3<f32>(0.2,       /* 1/5 */
                          0.14285,   /* 1/7 */
                          0.090090, /* 1/11 */),
                vec3<f32>(0.0),
                0.5
            )
        "
        }
        Colourer::Monochrome2 => {
            "simple_cos(2.0*log(max(iters-3.0, 1.0)), vec3(1.0), vec3(0.0), 0.5)"
        }
        Colourer::Neon2 => {
            "powered_cos( log(max(iters-3.0, 1.0)) * 2.0, vec3(0.0, 2.0*PI/3.0, 4.0*PI/3.0), vec3(-1.0), vec3(1.0), 1.5, 1.2 )"
        }
        Colourer::OneLoneCoder => {
            "simple_cos(
                iters * 0.1,
                vec3(1.0),
                vec3(
                    -FRAC_PI_2,
                    -FRAC_PI_2 + TAU / 3.0,
                    -FRAC_PI_2 + 2.0 * TAU / 3.0,
                ),
                0.5
            )"
        }
        Colourer::WhiteFade => {
            "simple_cos(log( max(iters-3.0, 1.0) ), vec3<f32>(2.0, 1.5, 1.0), vec3<f32>(0.0), 0.5)"
        }
        Colourer::None => "vec3(0.0, 0.0, 1.0)",
    }
}

#[must_use]
/// Assembles and returns the WGSL fragment shader code for the given shader key.
pub fn fragment_shader(key: ShaderKey) -> String {
    assemble_shader(ShaderType::Fragment, key.variant, key.colourer)
}

#[must_use]
/// Assembles and returns the WGSL compute shader code for the given shader key.
pub fn compute_shader(key: ShaderKey) -> String {
    assemble_shader(ShaderType::Compute, key.variant, key.colourer)
}

#[must_use]
/// Returns the WGSL shader code, either compute or fragment based on the flag.
/// Intended for debugging.
pub fn debug_output(key: ShaderKey, compute: bool) -> String {
    if compute {
        compute_shader(key)
    } else {
        fragment_shader(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_shader() {
        let shader = fragment_shader(ShaderKey {
            variant: ShaderVariant::General,
            colourer: Colourer::BlackFade,
        });
        assert!(shader.contains("fn main_fs"));
        assert!(shader.contains("fn main_vs"));
        assert!(shader.contains("Algorithm_Mandelbrot"));
        assert!(shader.contains("struct FragmentConstants"));
        println!("{shader}");
    }

    #[test]
    fn test_compute_shader() {
        let shader = compute_shader(ShaderKey {
            variant: ShaderVariant::General,
            colourer: Colourer::BlackFade,
        });
        assert!(shader.contains("fn main_cs"));
        assert!(shader.contains("Algorithm_Mandelbrot"));
        assert!(shader.contains("struct FragmentConstants"));
    }

    fn validate(src: &str) -> Result<(), String> {
        let module = naga::front::wgsl::parse_str(src).map_err(|e| e.emit_to_string(src))?;
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .map(|_| ())
        .map_err(|e| e.emit_to_string(src))
    }

    /// Validates both shader types for every value of one dimension, holding all other
    /// dimensions at their default. Dimensions are assumed independent, so there is no need
    /// to test the Cartesian product.
    fn check_dimension<T: std::fmt::Debug + Copy>(
        values: impl IntoIterator<Item = T>,
        apply: impl Fn(&mut ShaderKey, T),
    ) {
        for v in values {
            let mut key = ShaderKey::default();
            apply(&mut key, v);
            for (name, src) in [
                ("fragment", fragment_shader(key)),
                ("compute", compute_shader(key)),
            ] {
                validate(&src).unwrap_or_else(|e| panic!("{name} {v:?} ({key:?}):\n{e}"));
            }
        }
    }

    #[test]
    fn default_key_validates() {
        check_dimension([()], |_, ()| {});
    }

    #[test]
    fn variants_validate() {
        use strum::IntoEnumIterator;
        check_dimension(ShaderVariant::iter(), |k, v| k.variant = v);
    }

    #[test]
    fn colourers_validate() {
        use strum::IntoEnumIterator;
        check_dimension(Colourer::iter(), |k, v| k.colourer = v);
    }
}
