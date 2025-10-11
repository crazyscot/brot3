//! Fractal shader entrypoint

#![cfg_attr(target_arch = "spirv", no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

use shader_common::{FragmentConstants, GRID_SIZE, RenderData, complex::Complex};
use shader_util::grid::{GridRef, GridRefMut};

mod exponentiation;
use exponentiation::Exponentiator;

mod fractal;
use fractal::FractalImpl;

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
    // window-relative coords (0,W) x (0,H) (they might be half pixels e.g. 0.5 to 1023.5); we ignore depth & 1/w
    let coord = frag_coord.xy();
    // viewport pixel size e.g. 1920x1080
    let size = constants.size.as_vec2();

    let render_data = if constants.needs_reiterate.into() {
        // convert pixel coordinates to complex units such that (0,0) is at the centre of the viewport
        let cplx = (coord - 0.5 * size) / size.y / constants.viewport_zoom;
        let render_data = fractal::render(constants, cplx + constants.viewport_translate);
        let mut cache = GridRefMut::new(GRID_SIZE, grid);
        cache.set(coord.as_uvec2(), render_data);
        render_data
    } else {
        let cache = GridRef::new(GRID_SIZE, grid);
        cache.get(coord.as_uvec2())
    };

    let colour = colour_data(render_data, constants);

    *output = colour.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    // uv expresses the cycle: (0,0) (2,0) (0,2) (2,2)
    let pos = 2.0 * uv - Vec2::ONE;
    // pos expresses the cycle: (-1,-1) (3,-1) (-1,3) (3,3)

    *out_pos = pos.extend(0.0).extend(1.0);
}

struct Builder<'a, F, E>
where
    F: FractalImpl<E>,
    E: Exponentiator,
{
    constants: &'a FragmentConstants,
    f_impl: F,
    _phantom: core::marker::PhantomData<E>,
}

impl<F, E> Builder<'_, F, E>
where
    F: FractalImpl<E>,
    E: Exponentiator,
{
    fn iterations(self) -> RenderData {
        let FractalResult {
            inside,
            iters,
            smoothed_iters,
        } = self.f_impl.iterate(self.constants);
        RenderData::new(self.constants, inside, iters, smoothed_iters)
    }
}

struct FractalResult {
    /// is the pixel inside the set
    inside: bool,
    /// iteration count
    iters: u32,
    /// smoothed iteration count (where available)
    smoothed_iters: f32,
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

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{FragmentConstants, GRID_SIZE, RenderData};

    use shader_common::{Algorithm, PushExponent};
    use shader_util::Size;
    use spirv_std::glam::{Vec4, vec2, vec4};

    #[test]
    fn vertex() {
        let cases = &[
            (0, vec4(-1., -1., 0., 1.)),
            (1, vec4(3., -1., 0., 1.)),
            (2, vec4(-1., 3., 0., 1.)),
            (3, vec4(3., 3., 0., 1.)),
            (4, vec4(-1., -1., 0., 1.)),
        ];

        for (id, expected) in cases {
            let mut res = Vec4::default();
            super::main_vs(*id, &mut res);
            assert_eq!(&res, expected, "failing case: {id}");
        }
    }

    fn test_frag_consts() -> FragmentConstants {
        FragmentConstants {
            viewport_translate: vec2(0., 0.),
            viewport_zoom: 0.3,
            size: Size::new(1024, 1024),
            max_iter: 10,
            needs_reiterate: true.into(),
            algorithm: Algorithm::Mandelbrot,
            exponent: PushExponent::from(2),
        }
    }

    #[test]
    fn fragment() {
        let mut res = Vec4::default();
        let consts = test_frag_consts();
        let mut grid = vec![RenderData::default(); (GRID_SIZE.x * GRID_SIZE.y) as usize];
        super::main_fs(vec4(0., 0., 0., 0.), &consts, &mut grid, &mut res);
        let expected = vec4(1., 0.34425634, 0., 1.);
        assert!(
            res.abs_diff_eq(expected, 0.000_000_1),
            "mismatch: {res} vs {expected}"
        );
    }
}
