//! Benchmark harness for iai-callgrind
// (c) 2024 Ross Younger

#![allow(missing_docs)]

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;
use std::str::FromStr;

#[allow(clippy::enum_glob_use)]
#[allow(clippy::wildcard_imports)]
use brot3_engine::{
    colouring::{IColourer, Rgb8, direct_rgb::*, huecycles::*},
    fractal::{Algorithm, IAlgorithm, Point, PointData},
};

struct BenchData {
    point: PointData,
    fractal: Algorithm,
}

// ////////////////////////////////////////////////////////////////
// PREP

const PREP_POINT: Point = Point::new(0.1, 0.1);

/// Setup function for prepare
fn s_prep(alg: &str) -> BenchData {
    let point = PointData::new(PREP_POINT);
    let fractal = Algorithm::from_str(alg).unwrap_or_else(|_| panic!("can't find {alg}"));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(s_prep("mandelbrot"))]
#[bench::i2(s_prep("mandeldrop"))]
fn prepare(mut bd: BenchData) -> PointData {
    bd.fractal.prepare(&mut bd.point);
    bd.point
}

// ////////////////////////////////////////////////////////////////
// ITERATION

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

/// Setup function for iterate
fn s_iter(point_to_use: Point, alg: &str) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = Algorithm::from_str(alg).unwrap_or_else(|_| panic!("can't find {alg}"));
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::zero(s_iter(TEST_POINT_M2, "zero"))]
#[bench::m2(s_iter(TEST_POINT_M2, "mandelbrot"))]
#[bench::m3(s_iter(TEST_POINT_M3, "mandel3"))]
#[bench::bar(s_iter(TEST_POINT_M3, "mandelbar"))]
#[bench::ship(s_iter(TEST_POINT_M3, "burningship"))]
#[bench::celtic(s_iter(TEST_POINT_M3, "celtic"))]
#[bench::variant(s_iter(TEST_POINT_M3, "variant"))]
#[bench::bird(s_iter(TEST_POINT_M3, "bird"))]
#[bench::buffalo(s_iter(TEST_POINT_M3, "buffalo"))]
fn iterate(mut bd: BenchData) -> PointData {
    bd.fractal.iterate(&mut bd.point);
    bd.point
}

// ////////////////////////////////////////////////////////////////
// FINISH

/// Setup function for finish
fn s_fini(point_to_use: Point, alg: &str) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = Algorithm::from_str(alg).unwrap_or_else(|_| panic!("can't find {alg}"));
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

// CAUTION: When optimising the finish algorithm bear in mind that it generally runs the iteration a couple of times.
#[library_benchmark]
#[bench::m2(s_fini(TEST_POINT_M2, "mandelbrot"))]
#[bench::m3(s_fini(TEST_POINT_M3, "mandel3"))]
#[bench::ship(s_fini(TEST_POINT_M3, "burningship"))]
fn finish(mut bd: BenchData) -> Option<f32> {
    bd.fractal.finish(&mut bd.point);
    bd.point.result
}

// ////////////////////////////////////////////////////////////////

library_benchmark_group!(
    name = b_fractal;
    benchmarks = prepare, iterate, finish
);

// ////////////////////////////////////////////////////////////////
// COLOURING

use brot3_engine::colouring::{Colourer, testing::White};

#[library_benchmark]
#[bench::white(White {}.into())]
#[bench::linear_rainbow(LinearRainbow {}.into())]
#[bench::log_rainbow(LogRainbow {}.into())]
#[bench::mandy(Mandy {}.into())]
#[bench::white_fade(WhiteFade {}.into())]
#[bench::black_fade(BlackFade {}.into())]
#[bench::monochrome(Monochrome {}.into())]
#[bench::monochrome_inv(MonochromeInverted {}.into())]
#[bench::olc(OneLoneCoder {}.into())]
#[bench::hsv_grad(HsvGradient{}.into())]
#[bench::lch_grad(LchGradient{}.into())]
fn colour(alg: Colourer) -> Rgb8 {
    alg.colour_rgb8(black_box(42.0), 256)
}

library_benchmark_group!(
    name = b_colourer;
    benchmarks = colour,
);

main!(library_benchmark_groups = b_fractal, b_colourer);
