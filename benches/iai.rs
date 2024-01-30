use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

use brot3::{
    colouring::{direct_rgb, huecycles, ColourerInstance, OutputsRgb8, Rgb8},
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

fn setup_prep(alg: fractal::Selection) -> BenchData {
    let point = PointData::new(PREP_POINT);
    let fractal = fractal::factory(alg);
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(setup_prep(Original))]
#[bench::i2(setup_prep(Mandeldrop))]
fn bench_prep(mut bd: BenchData) -> PointData {
    bd.fractal.prepare(&mut bd.point);
    bd.point
}

// ////////////////////////////////////////////////////////////////
// ITERATION

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

fn setup_iteration(point_to_use: Point, alg: fractal::Selection) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = fractal::factory(alg);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(setup_iteration(TEST_POINT_M2, Original))]
#[bench::m3(setup_iteration(TEST_POINT_M3, Mandel3))]
#[bench::bar(setup_iteration(TEST_POINT_M3, Mandelbar))]
#[bench::ship(setup_iteration(TEST_POINT_M3, BurningShip))]
#[bench::celtic(setup_iteration(TEST_POINT_M3, Celtic))]
#[bench::variant(setup_iteration(TEST_POINT_M3, Variant))]
#[bench::bird(setup_iteration(TEST_POINT_M3, BirdOfPrey))]
#[bench::buffalo(setup_iteration(TEST_POINT_M3, Buffalo))]
fn bench_iteration(mut bd: BenchData) -> PointData {
    bd.fractal.iterate(&mut bd.point);
    bd.point
}

// ////////////////////////////////////////////////////////////////
// FINISH

fn setup_finish(point_to_use: Point, alg: fractal::Selection) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = fractal::factory(alg);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

// CAUTION: When optimising the finish algorithm bear in mind that it generally runs the iteration a couple of times.
#[library_benchmark]
#[bench::m2(setup_finish(TEST_POINT_M2, Original))]
#[bench::m3(setup_finish(TEST_POINT_M3, Mandel3))]
fn bench_finish(mut bd: BenchData) -> Option<fractal::Scalar> {
    bd.fractal.finish(&mut bd.point);
    bd.point.result
}

// ////////////////////////////////////////////////////////////////

library_benchmark_group!(
    name = iteration;
    benchmarks = bench_prep, bench_iteration, bench_finish
);

// ////////////////////////////////////////////////////////////////
// COLOURING

use direct_rgb::*;
use huecycles::*;

#[library_benchmark]
#[bench::linear_rainbow(LinearRainbow {}.into())]
#[bench::mandy(Mandy {}.into())]
fn bench_colourer(alg: ColourerInstance) -> Rgb8 {
    alg.colour_rgb8(black_box(42.0))
}

library_benchmark_group!(
    name = colourer;
    benchmarks = bench_colourer,
);

main!(library_benchmark_groups = iteration, colourer);
