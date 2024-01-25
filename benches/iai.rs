use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

use brot3::{
    colouring::{direct_rgb, huecycles, OutputsRgb8, PaletteInstance, Rgb8},
    fractal::{self, Algorithm, FractalInstance, Point, PointData},
};

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
//const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

struct BenchData {
    point: PointData,
    fractal: FractalInstance,
}

fn setup_iteration() -> BenchData {
    let mut point = PointData::new(TEST_POINT_M2);
    let fractal = fractal::factory(fractal::Selection::Original);
    fractal.prepare(black_box(&mut point));
    BenchData { point, fractal }
}

#[library_benchmark]
#[bench::short(setup_iteration())]
fn bench_iteration_m2(mut bd: BenchData) -> PointData {
    bd.fractal.iterate(&mut bd.point);
    bd.point
}

library_benchmark_group!(
    name = bench_iter_group;
    benchmarks = bench_iteration_m2
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

main!(library_benchmark_groups = bench_iter_group, colourer);
