use brot3_lib::ui::ShaderVariant;

const GENERAL_SHADER_BYTES: &[u8] = include_bytes!(concat!(
    env!("BROT3_PREBUILD_SHADERS_DIR"),
    "/brot3_general.spv"
));
const MANDELBROT_POW2_SHADER_BYTES: &[u8] = include_bytes!(concat!(
    env!("BROT3_PREBUILD_SHADERS_DIR"),
    "/brot3_mandelbrot_pow2.spv"
));
const MANDELBROT_POW2_DEEP_SHADER_BYTES: &[u8] = include_bytes!(concat!(
    env!("BROT3_PREBUILD_SHADERS_DIR"),
    "/brot3_mandelbrot_pow2_deep.spv"
));

pub(crate) const fn prebuilt_shader_bytes(variant: ShaderVariant) -> &'static [u8] {
    match variant {
        ShaderVariant::General => GENERAL_SHADER_BYTES,
        ShaderVariant::MandelbrotPow2 => MANDELBROT_POW2_SHADER_BYTES,
        ShaderVariant::MandelbrotPow2Deep => MANDELBROT_POW2_DEEP_SHADER_BYTES,
    }
}
