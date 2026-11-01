@vertex
fn main_vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    var out: VertexOutput;
    let position = positions[vertex_index];
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = position * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

const INSPECTOR_MARKER_SIZE: f32 = 9.0;
const INSPECTOR_MARKER_INNER_SIZE: f32 = INSPECTOR_MARKER_SIZE * 0.667;

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(f32(constants.viewport_size.width), f32(constants.viewport_size.height));
    let coord = vec2<f32>(in.uv.x * size.x, (1.0 - in.uv.y) * size.y);
    var rgb = mandelbrot_pixel(coord, size);
    if ((constants.flags.bits & FLAG_INSPECTOR_ACTIVE) != 0u) {
        let dist = new_york_distance(coord, constants.inspector_point_pixel_address);
        let outer_rgb = select(rgb, vec3<f32>(1.0, 1.0, 1.0), dist < INSPECTOR_MARKER_SIZE);
        rgb = select(outer_rgb, vec3<f32>(0.0, 0.0, 0.0), dist < INSPECTOR_MARKER_INNER_SIZE);
    }
    return vec4<f32>(rgb, 1.0);
}
fn new_york_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let delta = abs(a - b);
    return delta.x + delta.y;
}