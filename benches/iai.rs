use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

use brot3::{
    colouring::{direct_rgb, huecycles, OutputsRgb8, PaletteInstance, Rgb8},
    fractal::{self, Algorithm, FractalInstance, Point, PointData},
};

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

struct BenchData {
    point: PointData,
    fractal: FractalInstance,
}

use fractal::Selection::*;

fn setup_iteration(point_to_use: Point, alg: fractal::Selection) -> BenchData {
    let mut point = PointData::new(point_to_use);
    let fractal = fractal::factory(alg);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::m2(setup_iteration(TEST_POINT_M2, Original))]
#[bench::m3(setup_iteration(TEST_POINT_M3, Mandel3))]
fn bench_iteration(mut bd: BenchData) -> PointData {
    bd.fractal.iterate(&mut bd.point);
    bd.point
}

library_benchmark_group!(
    name = iteration;
    benchmarks = bench_iteration
);

///////////////////////////////////////////////////////////////////////////

use direct_rgb::*;
use huecycles::*;

#[library_benchmark]
#[bench::linear_rainbow(LinearRainbow {}.into())]
#[bench::mandy(Mandy {}.into())]
fn bench_colourer(alg: PaletteInstance) -> Rgb8 {
    alg.colour_rgb8(black_box(42.0))
}

library_benchmark_group!(
    name = colourer;
    benchmarks = bench_colourer,
);

main!(library_benchmark_groups = iteration, colourer);
