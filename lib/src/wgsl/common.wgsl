const FLAG_INSPECTOR_ACTIVE: u32 = 1<<1;
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
