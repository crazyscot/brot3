//! Fractal shader entrypoint

#![no_std]

use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

use engine3_common::{FragmentConstants, GRID_SIZE, RenderData, complex::Complex};
use shader_util::grid::{GridRef, GridRefMut};

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &mut [RenderData],
    output: &mut Vec4,
) {
    let coord = frag_coord.xy();
    let size = constants.size.as_vec2();
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5)
    // we ignore depth & 1/w

    // map xy coords to (-0.5,0.5) in both dimensions whilst applying an aspect ratio fix
    //let mut uv = (coord - 0.5 * size) / constants.size.height as f32;

    // (this will become panned and zoomed soon)
    //uv *= 2.0;

    let render_data = if constants.needs_reiterate.into() {
        let dc = (coord - 0.5 * size) / size.y;
        // both coords of dc are in range (-0.5,0.5); map this to the fixed locus (-2,2) for now
        // TODO zoom factor
        let dc = 2.5 * dc;
        let render_data = render_fractal(
            constants,
            Mandelbrot {
                z0: Complex::ZERO,
                c: dc.into(),
            },
        );
        let mut cache = GridRefMut::new(GRID_SIZE, grid);
        cache.set(coord.as_uvec2(), render_data);
        render_data
    } else {
        let cache = GridRef::new(GRID_SIZE, grid);
        cache.get(coord.as_uvec2())
    };

    let colour = colour_data(render_data, constants);

    *output = colour.powf(2.2).extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
    // This has the effect of generating a loop of the cycle (0,0,0,1) (2,0,0,1) (0,2,0,1) (2,2,0,1)
}

fn render_fractal(constants: &FragmentConstants, m: Mandelbrot) -> RenderData {
    let builder = Builder {
        constants,
        fractal: m,
    };
    builder.iterations()
}

struct Builder<'a> {
    constants: &'a FragmentConstants,
    fractal: Mandelbrot, // TODO this will become a trait obj
}

impl Builder<'_> {
    fn iterations(self) -> RenderData {
        let FractalResult { inside, iters } = self.fractal.iterate(self.constants);
        RenderData::new(self.constants, inside, iters)
    }
}

struct FractalResult {
    /// is the pixel inside the set
    inside: bool,
    /// iteration count
    iters: u32,
}

struct Mandelbrot {
    z0: Complex,
    c: Complex,
}

impl Mandelbrot {
    fn iterate(self, _constants: &FragmentConstants) -> FractalResult {
        const ESCAPE_THRESHOLD_SQ: f32 = 4.0;
        const ITER_LIMIT: u32 = 256;

        let mut iters = 0;
        let mut z = self.z0;
        let c = self.c;
        let mut norm_sqr = z.abs_sq();
        // TODO: Cardoid and period-2 bulb checks?

        while norm_sqr < ESCAPE_THRESHOLD_SQ && iters < ITER_LIMIT {
            z = z * z + c;
            iters += 1;
            norm_sqr = z.abs_sq();
        }
        let inside = iters == ITER_LIMIT && (norm_sqr < ESCAPE_THRESHOLD_SQ);
        FractalResult { inside, iters }
    }
}

fn colour_data(data: RenderData, _constants: &FragmentConstants) -> Vec3 {
    let RenderData {
        iters,
        smooth_iters,
    } = data;
    if iters == u32::MAX {
        return Vec3::ZERO;
    }
    log_rainbow(smooth_iters)
}

/// quick and dirty hsv to rgb conversion for now
fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    fn is_between(value: f32, min: f32, max: f32) -> bool {
        min <= value && value < max
    }

    let c = value * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = value - c;

    let (r, g, b): (f32, f32, f32) = if is_between(h, 0.0, 1.0) {
        (c, x, 0.0)
    } else if is_between(h, 1.0, 2.0) {
        (x, c, 0.0)
    } else if is_between(h, 2.0, 3.0) {
        (0.0, c, x)
    } else if is_between(h, 3.0, 4.0) {
        (0.0, x, c)
    } else if is_between(h, 4.0, 5.0) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Vec3::new(r + m, g + m, b + m)
}

fn log_rainbow(t: f32) -> Vec3 {
    let angle = t.ln() * 60.0; // DEGREES
    hsv_to_rgb(angle, 1.0, 1.0)
}
