
fn colour_pixel(data: PointResult) -> vec3<f32> {
    let iters = f32(data.iters) + data.iters_fraction;
    return colour_impl(data, iters); // Provided by colour fragment
}

fn scaled_palette_offset() -> f32 {
    // Offset is applied before cos(), so scale the input (0..10) to 2pi.
    return constants.palette.offset * TAU / 10.0;
}

fn simple_cos(processed_iters: f32, vec_scale: vec3<f32>, vec_offset: vec3<f32>, cos_factor: f32) -> vec3<f32> {
    var v: vec3<f32> = vec_scale * processed_iters * constants.palette.gradient + vec3<f32>(scaled_palette_offset()) + vec_offset;
    v = cos(v) * cos_factor + vec3<f32>(0.5);
    return v;
}

fn powered_cos(processed_iters: f32, vec_offset: vec3<f32>, pow_factor: vec3<f32>, pow_offset: vec3<f32>, power: f32, gamma: f32) -> vec3<f32> {
    var v = vec3(pow(processed_iters * constants.palette.gradient, gamma)) + scaled_palette_offset() + vec_offset;
    v = cos(v) * 0.5 + vec3(0.5);
    v = pow(v, vec3(power));
    v = v * pow_factor + pow_offset;
    return v;
}