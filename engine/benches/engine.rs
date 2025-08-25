//! brot3 engine benchmarking harness
// (c) 2024 Ross Youinger

#![allow(missing_docs)]

use brot3_engine::{
    colouring::{Colourer, IColourer, huecycles::LinearRainbow},
    fractal::{Algorithm, IAlgorithm, Location, Point, PointData, Size, Tile, TileSpec},
    render::Png,
    util::Rect,
};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::str::FromStr as _;

// //////////////////////////////////////////////////////////////////////////////////////////
// FRACTALS

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);
const TEST_COLOURER: Colourer = Colourer::LinearRainbow(LinearRainbow {});

fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("fractals");
    let mut alg = |alg, point: Point| {
        let fractal = Algorithm::from_str(alg).unwrap_or_else(|_| panic!("can't find {alg}"));
        let _ = group.bench_function(format!("iter_{alg}"), |b| {
            b.iter_batched_ref(
                || {
                    let mut pd = PointData::new(point);
                    fractal.prepare(&mut pd);
                    assert!(pd.result.is_none());
                    pd
                },
                |pd| fractal.iterate(black_box(pd)),
                BatchSize::SmallInput,
            );
        });
    };
    alg("mandelbrot", TEST_POINT_M2);
    alg("mandel3", TEST_POINT_M3);
    alg("mandelbar", TEST_POINT_M3);
    alg("variant", TEST_POINT_M3);
    alg("zero", TEST_POINT_M3);
}

fn get_test_tile_spec(alg: Algorithm, dimension: u32) -> TileSpec {
    TileSpec::new(
        Location::Origin(Point { re: -1.0, im: 0.0 }),
        Size::AxesLength(Point { re: 4.0, im: 4.0 }),
        Rect::new(dimension, dimension),
        alg,
        512,
        TEST_COLOURER,
    )
}

fn plot_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let mut do_alg = |alg| {
        let fractal = Algorithm::from_str(alg).unwrap();
        let spec = get_test_tile_spec(fractal, 100);
        let _ = group.bench_function(format!("plot_{alg}"), |b| {
            b.iter_batched_ref(
                || Tile::new(&spec, 0),
                |t| black_box(t).plot(),
                BatchSize::SmallInput,
            );
        });
    };
    do_alg("mandelbrot");
    do_alg("zero");
}

criterion_group!(fractals, iteration, plot_tile);

// //////////////////////////////////////////////////////////////////////////////////////////
// COLOURING

fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("colourers");
    let mut bench = |instance: Colourer| {
        let _ = group.bench_function(format!("{instance}"), |b| {
            b.iter(|| {
                let _ = instance.colour_rgb8(black_box(42.0), 256);
            });
        });
    };
    // We run only a selection of algorithms through the full benchmarker
    // (See also IAI, which runs them all.)
    let selection = ["linear-rainbow", "lch-gradient", "mandy", "white-fade"];
    for i in &selection {
        let it = Colourer::from_str(i).unwrap_or_else(|_| panic!("can't find {}", *i));
        bench(it);
    }
}

fn colour_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let alg = Algorithm::from_str("mandelbrot").unwrap();
    let spec = get_test_tile_spec(alg, 100);
    let mut tile = Tile::new(&spec, 0);
    tile.plot();

    let mut bench = |colourer: Colourer| {
        let _ = group.bench_function(format!("colour_{colourer}"), |b| {
            b.iter(|| {
                // TODO use render_rgba_into()
                let _ = black_box(Png::render_rgba(black_box(&tile), colourer));
            });
        });
    };
    let selection = ["linear-rainbow", "white"];
    for i in &selection {
        let it = Colourer::from_str(i).unwrap_or_else(|_| panic!("can't find {alg}"));
        bench(it);
    }
}

criterion_group!(colourers, colour_pixel, colour_tile);

// //////////////////////////////////////////////////////////////////////////////////////////

criterion_main!(fractals, colourers);
