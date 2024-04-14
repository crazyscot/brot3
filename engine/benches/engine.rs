//! brot3 engine benchmarking harness
// (c) 2024 Ross Youinger

#![allow(missing_docs)]

#[allow(clippy::enum_glob_use)]
use brot3_engine::{
    colouring::{self, Instance, OutputsRgb8, Selection::*},
    fractal::{self, Algorithm, Location, Point, PointData, Size, SplitMethod, Tile, TileSpec},
    render::Png,
    util::Rect,
};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

// //////////////////////////////////////////////////////////////////////////////////////////
// FRACTALS

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);
const TEST_COLOURER: colouring::Selection = colouring::Selection::LinearRainbow;

fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("fractals");
    let mut alg = |alg, point: Point| {
        let fractal = fractal::factory(alg);
        let _ = group.bench_function(format!("iter_{alg:?}"), |b| {
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
    alg(fractal::Selection::Original, TEST_POINT_M2);
    alg(fractal::Selection::Mandel3, TEST_POINT_M3);
    alg(fractal::Selection::Mandelbar, TEST_POINT_M3);
    alg(fractal::Selection::Variant, TEST_POINT_M3);
    alg(fractal::Selection::Zero, TEST_POINT_M3);
}

fn get_test_tile_spec(alg: fractal::Selection, dimension: u32) -> TileSpec {
    TileSpec::new(
        Location::Origin(Point { re: -1.0, im: 0.0 }),
        Size::AxesLength(Point { re: 4.0, im: 4.0 }),
        Rect::new(dimension, dimension),
        fractal::factory(alg),
        512,
        colouring::factory(TEST_COLOURER),
    )
}

fn plot_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let mut do_alg = |alg| {
        let spec = get_test_tile_spec(alg, 100);
        let _ = group.bench_function(format!("plot_{alg:?}"), |b| {
            b.iter_batched_ref(
                || Tile::new(&spec, 0),
                |t| black_box(t).plot(),
                BatchSize::SmallInput,
            );
        });
    };
    do_alg(fractal::Selection::Original);
    do_alg(fractal::Selection::Zero);
}

criterion_group!(fractals, iteration, plot_tile);

// //////////////////////////////////////////////////////////////////////////////////////////
// COLOURING

fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("colourers");
    let mut bench = |instance: Instance| {
        let _ = group.bench_function(format!("{instance}"), |b| {
            b.iter(|| {
                let _ = instance.colour_rgb8(black_box(42.0), 256);
            });
        });
    };
    // We run only a selection of algorithms through the full benchmarker
    // (See also IAI, which runs them all.)
    let selection = [LinearRainbow, LchGradient, Mandy, WhiteFade];
    selection.iter().for_each(|i| bench(colouring::factory(*i)));
}

fn colour_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let spec = get_test_tile_spec(fractal::Selection::Original, 100);
    let mut tile = Tile::new(&spec, 0);
    tile.plot();

    let mut bench = |colourer: Instance| {
        let _ = group.bench_function(format!("colour_{colourer}"), |b| {
            b.iter(|| {
                let _ = black_box(Png::render_rgba(black_box(&tile), colourer));
            });
        });
    };
    let selection = [LinearRainbow, White];
    selection.iter().for_each(|i| bench(colouring::factory(*i)));
}

criterion_group!(colourers, colour_pixel, colour_tile);

// //////////////////////////////////////////////////////////////////////////////////////////
// TILE OPERATIONS

fn tile_join(c: &mut Criterion) {
    // prepare and iterate on a bunch of tiles; we only care about the joining
    let mut group = c.benchmark_group("tiles");
    let single = get_test_tile_spec(fractal::Selection::Original, 1000);
    let specs = single.split(SplitMethod::RowsOfHeight(50), 0).unwrap();
    let mut tiles: Vec<_> = specs.iter().map(|ts| Tile::new(ts, 0)).collect();
    tiles.par_iter_mut().for_each(|t| black_box(t).plot());

    let _ = group.bench_function("join", |b| {
        b.iter(|| Tile::join(&single, black_box(&tiles)).unwrap());
    });
}

criterion_group!(tiles, tile_join);

// //////////////////////////////////////////////////////////////////////////////////////////

criterion_main!(fractals, colourers, tiles);
