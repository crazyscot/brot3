//! Cycle-based benchmarking

#![allow(missing_docs)]

use std::{hint::black_box, sync::LazyLock};

use brot3_lib::{
    BigVec2, Complex,
    data::{Algorithm, Colourer, FragmentConstants, PointResult},
    engine::{
        self, RunningConstants, RunningVariables, mandelbrot_family_iterate_algorithm,
        mandelbrot_perturbed_iterate_algorithm,
    },
    maths::{ComplexPower, Exponentiator, Power2, Power3, Power4, Power5, Power6},
    ui::UiState,
    util::Size,
};
use glam::{Vec2, vec2};
use gungraun::{library_benchmark, library_benchmark_group, main};
use num_traits::cast::AsPrimitive as _;

// ..........................................................

fn it_setup<E: Exponentiator>(e: E) -> RunningConstants<'static, E> {
    RunningConstants::standard_with(Complex::new(-0.75, 0.0), e, Algorithm::Mandelbrot)
}

fn it_alg_setup(algorithm: Algorithm) -> RunningConstants<'static, Power2> {
    RunningConstants::standard_with(Complex::new(-0.75, 0.0), Power2 {}, algorithm)
}

#[library_benchmark]
#[bench::special2(&it_setup(Power2{}))]
#[bench::special3(&it_setup(Power3{}))]
#[bench::special6(&it_setup(Power6{}))]
#[bench::complex2(&it_setup(ComplexPower::new(2.0, 0.0)))]
#[bench::complex3(&it_setup(ComplexPower::new(3.0, 0.0)))]
#[bench::complex20(&it_setup(ComplexPower::new(20.0, 0.0)))]
#[bench::complex2101i(&it_setup(ComplexPower::new(2.1, 0.1)))]
#[bench::minus2(&it_setup(ComplexPower::new(-2.0, 0.0)))]
#[bench::minus1(&it_setup(ComplexPower::new(-1.0, 0.0)))]
#[bench::zero(&it_setup(ComplexPower::new(0.0, 0.0)))]
#[bench::mbar(&it_alg_setup(Algorithm::Mandelbar))] // same result as m2
//#[bench::bird(&it_alg_setup(Algorithm::BirdOfPrey))] // same result as m2
fn iterate_std<E: Exponentiator>(consts: &RunningConstants<'_, E>) {
    let mut vars = RunningVariables::default();
    mandelbrot_family_iterate_algorithm(black_box(consts), black_box(&mut vars), black_box(0));
}

static CONSTS_M2_DEFAULT: LazyLock<FragmentConstants> = LazyLock::new(|| FragmentConstants {
    size: Size::new(800, 600),
    viewport_translate: Vec2::new(-1.0, 0.0),
    ..Default::default()
});

#[allow(clippy::cast_precision_loss)]
fn prep_render(x: u32, y: u32, constants: &FragmentConstants) -> (Vec2, FragmentConstants) {
    let coord = vec2(x as f32 + 0.5, y as f32 + 0.5);
    let size = constants.size.as_vec2();
    let pixel_spacing = constants.pixel_spacing();
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;
    (complex_offset, *constants)
}

#[library_benchmark]
#[bench::small_iters(prep_render(40, 58, &CONSTS_M2_DEFAULT))] // iters = 2
#[bench::large_iters(prep_render(539,200, &CONSTS_M2_DEFAULT))] // iters = 97
#[bench::in_set(prep_render(497,311, &CONSTS_M2_DEFAULT))] // the full default 250
fn render_point(params: (Vec2, FragmentConstants)) {
    let (complex_offset, constants) = params;
    let render_data = engine::render(
        &constants,
        complex_offset,
        &[], // perturbation reference points
    );
    let _ = black_box(render_data);
}

const MAXITER_REFPOINTS: usize = 250;

fn prep_bigvec2(filename: &str) -> BigVec2 {
    let state = UiState::load_json(&filename).expect("Failed to load state");
    state.viewport_translate
}

#[library_benchmark]
#[bench::in_set(&prep_bigvec2("../data/m2-minibrot-perturbation-neon.json"))]
fn reference_points(centre: &BigVec2) {
    let mut points = Vec::with_capacity(MAXITER_REFPOINTS);
    // Interestingly, we don't need to know the zoom to compute the reference points!
    engine::mandelbrot_perturbed_compute_reference_iters(
        &mut points,
        centre,
        Algorithm::Mandelbrot,
        MAXITER_REFPOINTS.as_(),
    );
}

struct PerturbedSetup<'a> {
    constants: RunningConstants<'a, Power2>,
    reference_points: Vec<Vec2>,
}

impl PerturbedSetup<'_> {
    fn new(filename: &str, x: u32, y: u32) -> PerturbedSetup<'_> {
        let state = UiState::load_json(&filename).expect("Failed to load state");

        let mut reference_points = Vec::with_capacity(MAXITER_REFPOINTS);
        engine::mandelbrot_perturbed_compute_reference_iters(
            &mut reference_points,
            &state.viewport_translate,
            Algorithm::Mandelbrot,
            MAXITER_REFPOINTS.as_(),
        );

        // viewport pixel size e.g. 1920x1080
        let viewport_size = vec2(800.0, 600.0);
        // pixel address within the viewport
        let pixel_address = vec2(x.as_(), y.as_());
        let constants = FragmentConstants::from(&state);
        // convert pixel coordinates to complex units such that (0,0) is at the centre of the
        // viewport
        let complex_offset = (pixel_address - 0.5 * viewport_size) * constants.pixel_spacing();

        PerturbedSetup {
            constants: RunningConstants::perturbed_with(
                state.viewport_translate.as_vec2().into(),
                Power2 {},
                Algorithm::Mandelbrot,
                complex_offset.into(),
            ),
            reference_points,
        }
    }
}

#[library_benchmark]
#[bench::deepzoom(&PerturbedSetup::new("../data/m2-minibrot-perturbation-neon.json", 322, 246))]
fn iterate_perturbed(setup: &PerturbedSetup<'_>) {
    let mut vars = RunningVariables::default();
    let mut consts = setup.constants;
    consts.n_reference = setup.reference_points.len();
    consts.reference_points = &setup.reference_points;
    // This is a single iteration, so the numbers are quite small.
    mandelbrot_perturbed_iterate_algorithm(black_box(&consts), black_box(&mut vars), black_box(0));
}

library_benchmark_group!(
    name = fractal,
    benchmarks = [
        iterate_std,
        render_point,
        reference_points,
        iterate_perturbed
    ]
);

// ..........................................................

#[allow(clippy::cast_precision_loss)]
fn col(colourer: Colourer) -> FragmentConstants {
    let mut consts = *CONSTS_M2_DEFAULT;
    consts.palette.colourer = colourer;
    consts
}

static COLOUR_DATA: LazyLock<PointResult> = LazyLock::new(|| {
    PointResult::new(
        9,
        0.929,
        -1.249,
        10.367,
        brot3_lib::data::BoundaryClass::NotClose,
    )
});

#[library_benchmark]
#[bench::mono(col(Colourer::Monochrome), &COLOUR_DATA)]
#[bench::icyblue(col(Colourer::IcyBlue), &COLOUR_DATA)]
#[bench::olc(col(Colourer::OneLoneCoder), &COLOUR_DATA)]
#[bench::log_rainbow(col(Colourer::LogRainbow), &COLOUR_DATA)]
#[bench::neon(col(Colourer::Neon), &COLOUR_DATA)]
#[bench::blackfade(col(Colourer::BlackFade), &COLOUR_DATA)]
#[bench::whitefade(col(Colourer::WhiteFade), &COLOUR_DATA)]
#[bench::mandy(col(Colourer::Mandy), &COLOUR_DATA)]
fn colour_pt(consts: FragmentConstants, data: &PointResult) {
    let spacing = consts.pixel_spacing();
    let colour = engine::colour_data(black_box(*data), black_box(&consts), black_box(spacing));
    let _ = black_box(colour);
}

#[library_benchmark]
fn hsl_to_rgb() {
    let hsl = brot3_lib::util::Hsl::new(128.0, 100.0, 100.0);
    let _ = black_box(brot3_lib::util::RgbVec::from(black_box(hsl)));
}

library_benchmark_group!(name = colour, benchmarks = [colour_pt, hsl_to_rgb]);

// ..........................................................

#[library_benchmark]
#[bench::p2(Power2{})]
#[bench::p3(Power3{})]
#[bench::p4(Power4{})]
#[bench::p5(Power5{})]
#[bench::p6(Power6{})]
#[bench::comp2(ComplexPower::new(2.0, 0.0))]
#[bench::comp3(ComplexPower::new(3.0, 0.0))]
#[bench::comp6(ComplexPower::new(6.0, 0.0))]
#[bench::comp10(ComplexPower::new(10.0, 0.0))]
#[bench::comp20(ComplexPower::new(20.0, 0.0))]
#[bench::comp2101i(ComplexPower::new(2.1, 0.1))]
#[bench::comp2020(ComplexPower::new(20.0, 20.0))]
#[bench::comp_m2(ComplexPower::new(-2.0, 0.0))]
#[bench::comp_m1(ComplexPower::new(-1.0, 0.0))]
#[bench::comp_zero(ComplexPower::new(0.0, 0.0))]
#[bench::comp1(ComplexPower::new(1.0, 0.0))]
#[bench::comp_m20(ComplexPower::new(-20.0, 0.0))]
#[bench::comp_m2020(ComplexPower::new(-20.0, 20.0))]
#[bench::comp_m20m20(ComplexPower::new(-20.0, -20.0))]
fn exponent<E: Exponentiator>(e: E) {
    let z = Complex::new(0.5, -0.5);
    let _ = black_box(e.apply_to(black_box(z)));
}

#[library_benchmark]
#[bench::one(1)]
#[bench::two(2)]
fn is_multiple_of(i: u32) {
    let _ = black_box(i).is_multiple_of(black_box(2));
}

fn imo2_bithack(i: u32) -> bool {
    (i & 1) == 0
}

#[library_benchmark]
#[bench::one(1)]
#[bench::two(2)]
fn is_multiple_of_bithack(i: u32) {
    let _ = black_box(imo2_bithack(black_box(i)));
}

library_benchmark_group!(
    name = maths,
    benchmarks = [exponent, is_multiple_of, is_multiple_of_bithack,]
);

// ..........................................................

main!(library_benchmark_groups = fractal, colour, maths);
