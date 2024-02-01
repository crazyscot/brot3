// brot3 library benchmarking
// (c) 2024 Ross Youinger

use brot3::colouring::{self, Instance, OutputsRgb8, Selection::*};
use brot3::fractal::{self, Algorithm, Point, PointData, Tile, TileSpec};

use brot3::util::Rect;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

// //////////////////////////////////////////////////////////////////////////////////////////
// FRACTALS

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("fractals");
    let mut alg = |alg, point: Point| {
        let fractal = fractal::factory(alg);
        group.bench_function(format!("iter_{alg:?}"), |b| {
            b.iter_batched_ref(
                || {
                    let mut pd = PointData::new(point);
                    fractal.prepare(&mut pd);
                    pd
                },
                |pd| fractal.iterate(pd),
                BatchSize::SmallInput,
            );
        });
    };
    alg(fractal::Selection::Original, TEST_POINT_M2);
    alg(fractal::Selection::Mandel3, TEST_POINT_M3);
    alg(fractal::Selection::Mandelbar, TEST_POINT_M3);
    alg(fractal::Selection::Variant, TEST_POINT_M3);
    alg(fractal::Selection::Zero, TEST_POINT_M3);
}

fn get_test_tile_spec(alg: fractal::Selection) -> TileSpec {
    TileSpec::new(
        Point { re: -1.0, im: 0.0 },
        Point { re: 4.0, im: 4.0 },
        Rect::new(300, 300),
        fractal::factory(alg),
    )
}

fn tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let mut do_alg = |alg| {
        let spec = get_test_tile_spec(alg);
        group.bench_function(format!("tile_{alg:?}"), |b| {
            b.iter_batched_ref(
                || Tile::new(&spec, 0),
                |t| t.plot(black_box(512)),
                BatchSize::SmallInput,
            );
        });
    };
    do_alg(fractal::Selection::Original);
    do_alg(fractal::Selection::Zero);
}

criterion_group!(fractals, iteration, tile);

// //////////////////////////////////////////////////////////////////////////////////////////
// COLOURING

fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("colourers");
    let mut bench = |instance: Instance| {
        group.bench_function(format!("{}", instance), |b| {
            b.iter(|| {
                instance.colour_rgb8(black_box(42.0), 256);
            });
        });
    };
    // We run only a selection of algorithms through the full benchmarker
    // (See also IAI, which runs them all.)
    let selection = [LinearRainbow, LchGradient, Mandy, WhiteFade];
    selection
        .iter()
        .for_each(|i| bench(colouring::Instance::from_repr(*i as usize).unwrap()));
}

criterion_group!(colourers, colour_pixel);
criterion_main!(fractals, colourers);
