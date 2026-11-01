
@group(1) @binding(0)
var<storage, read_write> pixels: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn main_cs(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    let width = constants.viewport_size.width;
    let height = constants.viewport_size.height;
    if x >= width || y >= height {
        return;
    }

    let coord = vec2<f32>(f32(x), f32(y));
    let size = vec2<f32>(f32(width), f32(height));

    let rgb = mandelbrot_pixel(coord, size);

    let red = u32(rgb.x * 255.0);
    let green = u32(rgb.y * 255.0);
    let blue = u32(rgb.z * 255.0);

    let ix = y * width + x;
    pixels[ix] = (255u << 24u) | (blue << 16u) | (green << 8u) | red;
}