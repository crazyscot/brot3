const FLAG_INSPECTOR_ACTIVE: u32 = 1<<1;
const FLAG_ITERATION_CULL: u32 = 1<<3;
// TODO: Auto-sync these flag definitions with flags.rs
// (there isn't a tidy way to do this right now; maybe a second-order macro, maybe we ditch bitflags)

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;
const FRAC_PI_2: f32 = 1.5707963;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

var<immediate> constants: FragmentConstants;

@group(0) @binding(0)
var<storage, read> reference_points: array<vec2<f32>>;

// Many years ago, my previous fractal plotter `brot2` had a bug where
// the number of iterations was incorrectly clamped between passes.
//
// The effect was serendipitous: it increased the colour contrast and gradient
// in some regions (close to the set, where the number of iterations > 256:
// typically found at zooms of factor 1000x or more).
//
// This function reimplements that effect.
fn cull_iterations(iters_in: u32) -> u32 {
    // k := floor(log2(floor(iters / 640) + 1))
    let pair_index = iters_in / 640u + 1u;
    let k = 31u - countLeadingZeros(pair_index);
    // r := iters - 640 * (2^k - 1)
    let scale = 1u << k;
    let r = iters_in - 640u * (scale - 1u);
    let first_pass = 256u * scale;
    if (r < first_pass) {
        return r;
    }
    return r - first_pass;
}
