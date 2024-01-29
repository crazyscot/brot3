use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

use brot3::{
    colouring::{self, direct_rgb, huecycles, OutputsRgb8, Rgb8},
    fractal::{self, Algorithm, Point, PointData},
};
use fractal::Selection::*;

struct BenchData {
    point: PointData,
    fractal: fractal::Instance,
}

// ////////////////////////////////////////////////////////////////
// PREP

const PREP_POINT: Point = Point::new(0.1, 0.1);

/// Setup function for prepare
fn s_prep(alg: fractal::Selection) -> BenchData {
    let point = PointData::new(PREP_POINT);
    let fractal = fractal::factory(alg);
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(s_prep(Original))]
#[bench::i2(s_prep(Mandeldrop))]
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
fn s_iter(point_to_use: Point, alg: fractal::Selection) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = fractal::factory(alg);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(s_iter(TEST_POINT_M2, Original))]
#[bench::m3(s_iter(TEST_POINT_M3, Mandel3))]
#[bench::bar(s_iter(TEST_POINT_M3, Mandelbar))]
#[bench::ship(s_iter(TEST_POINT_M3, BurningShip))]
#[bench::celtic(s_iter(TEST_POINT_M3, Celtic))]
#[bench::variant(s_iter(TEST_POINT_M3, Variant))]
#[bench::bird(s_iter(TEST_POINT_M3, BirdOfPrey))]
#[bench::buffalo(s_iter(TEST_POINT_M3, Buffalo))]
fn iterate(mut bd: BenchData) -> PointData {
    bd.fractal.iterate(&mut bd.point);
    bd.point
}

// ////////////////////////////////////////////////////////////////
// FINISH

/// Setup function for finish
fn s_fini(point_to_use: Point, alg: fractal::Selection) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = fractal::factory(alg);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

// CAUTION: When optimising the finish algorithm bear in mind that it generally runs the iteration a couple of times.
#[library_benchmark]
#[bench::m2(s_fini(TEST_POINT_M2, Original))]
#[bench::m3(s_fini(TEST_POINT_M3, Mandel3))]
fn finish(mut bd: BenchData) -> Option<fractal::Scalar> {
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

use direct_rgb::*;
use huecycles::*;

#[library_benchmark]
#[bench::linear_rainbow(LinearRainbow {}.into())]
#[bench::log_rainbow(LogRainbow {}.into())]
#[bench::mandy(Mandy {}.into())]
#[bench::white_fade(WhiteFade {}.into())]
#[bench::black_fade(BlackFade {}.into())]
#[bench::monochrome(Monochrome {}.into())]
#[bench::monochrome_inv(MonochromeInverted {}.into())]
#[bench::olc(OneLoneCoder {}.into())]
fn colour(alg: colouring::Instance) -> Rgb8 {
    alg.colour_rgb8(black_box(42.0), 256)
}

library_benchmark_group!(
    name = b_colourer;
    benchmarks = colour,
);

main!(library_benchmark_groups = b_fractal, b_colourer);
