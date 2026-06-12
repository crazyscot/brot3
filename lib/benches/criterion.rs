#![allow(missing_docs)]

use std::{hint::black_box, sync::LazyLock};

use brot3_lib::{
    BigVec2,
    data::{Algorithm, FragmentConstants, PointResult},
    engine,
    engine::{RunningConstants, RunningVariables, mandelbrot_perturbed_iterate_algorithm},
    maths::Power2,
    ui::UiState,
    util::Size,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use easy_cast::{Cast as _, CastApprox as _, ConvApprox as _};
use glam::{Vec2, vec2};

static CONSTS_M2_DEFAULT: LazyLock<FragmentConstants> = LazyLock::new(|| FragmentConstants {
    size: Size::new(800, 600),
    viewport_translate: Vec2::new(-1.0, 0.0),
    ..Default::default()
});

fn prep_render(x: u32, y: u32, constants: &FragmentConstants) -> (Vec2, FragmentConstants) {
    let coord = vec2(f32::conv_approx(x) + 0.5, f32::conv_approx(y) + 0.5);
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;
    (complex_offset, *constants)
}

fn do_iterate(params: (Vec2, FragmentConstants)) -> PointResult {
    let (complex_offset, constants) = params;
    engine::render(
        &constants,
        complex_offset,
        &[], // perturbation reference points
    )
}

fn iterate_standard(c: &mut Criterion) {
    let params = prep_render(497, 311, &CONSTS_M2_DEFAULT);
    let _ = c.bench_with_input(
        BenchmarkId::new("do_iterate", "497, 311"),
        &params,
        |b, &s| {
            b.iter(|| do_iterate(s));
        },
    );
}

// ..................................................

fn prep_bigvec2(filename: &str) -> BigVec2 {
    let state = UiState::load_json(&filename).expect("Failed to load state");
    state.viewport_translate
}

const MAXITER_REFPOINTS: usize = 250;

fn reference_points(c: &mut Criterion) {
    let centre = prep_bigvec2("../data/m2-minibrot-perturbation-neon.json");
    let mut points = Vec::with_capacity(MAXITER_REFPOINTS);
    let _ = c.bench_with_input(
        BenchmarkId::new("refpoints", "m2-minibrot-perturbation-neon.json"),
        &centre,
        |b, s| {
            b.iter(|| {
                engine::mandelbrot_perturbed_compute_reference_iters(
                    &mut points,
                    s,
                    Algorithm::Mandelbrot,
                    MAXITER_REFPOINTS.cast(),
                );
            });
        },
    );
}

// ..................................................

fn iterate_perturbed(c: &mut Criterion) {
    let position = (322, 246);
    let state = UiState::load_json(&"../data/m2-minibrot-perturbation-neon.json")
        .expect("Failed to load state");
    let mut reference_points = Vec::with_capacity(MAXITER_REFPOINTS);
    engine::mandelbrot_perturbed_compute_reference_iters(
        &mut reference_points,
        &state.viewport_translate,
        Algorithm::Mandelbrot,
        MAXITER_REFPOINTS.cast(),
    );

    // viewport pixel size e.g. 1920x1080
    let viewport_size = vec2(800.0, 600.0);
    // pixel address within the viewport
    let pixel_address = vec2(position.0.cast_approx(), position.1.cast_approx());
    let frag_consts = FragmentConstants::from(&state);
    // convert pixel coordinates to complex units such that (0,0) is at the centre of the
    // viewport
    let complex_offset = (pixel_address - 0.5 * viewport_size) * frag_consts.pixel_spacing();

    let mut running_consts = RunningConstants::perturbed_with(
        state.viewport_translate.as_vec2().into(),
        Power2 {},
        Algorithm::Mandelbrot,
        complex_offset.into(),
        &[],
    );
    running_consts.reference_points = reference_points.as_slice();
    let vars = RunningVariables::default();

    let _ = c.bench_with_input(
        BenchmarkId::new("deep_iterate", "m2-minibrot-perturbation-neon.json"),
        &running_consts,
        |b, s| {
            b.iter(|| {
                let mut vars = vars;
                let mut z = brot3_lib::Complex::ZERO;
                // This is a single iteration, so the numbers are quite small.
                mandelbrot_perturbed_iterate_algorithm(
                    black_box(s),
                    black_box(&mut z),
                    black_box(&mut vars),
                    0,
                );
                let _ = black_box(vars);
            });
        },
    );
}

// ..................................................

criterion_group!(
    fractal,
    iterate_standard,
    reference_points,
    iterate_perturbed
);

criterion_main!(fractal);
