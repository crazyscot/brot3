//! brot3 engine benchmarking harness
// (c) 2024 Ross Youinger

#![allow(missing_docs)]

#[allow(clippy::enum_glob_use)]
use brot3_engine::{
    colouring::{self, Instance, OutputsRgb8, Selection::*},
    fractal::{self, Algorithm, Point, PointData, SplitMethod, Tile, TileSpec},
    render::{self, Renderer},
    util::Rect,
};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

// //////////////////////////////////////////////////////////////////////////////////////////
// FRACTALS

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT_M2: Point = Point::new(-0.158_653_6, 1.034_804);
const TEST_POINT_M3: Point = Point::new(-0.573_133_7, 0.569_299_8);

fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("fractals");
    let mut alg = |alg, point: Point| {
        let fractal = fractal::factory(alg);
        let _ = group.bench_function(format!("iter_{alg:?}"), |b| {
            b.iter_batched_ref(
                || {
                    let mut pd = PointData::new(point);
                    fractal.prepare(&mut pd);
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
        Point { re: -1.0, im: 0.0 },
        Point { re: 4.0, im: 4.0 },
        Rect::new(dimension, dimension),
        fractal::factory(alg),
    )
}

fn plot_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let mut do_alg = |alg| {
        let spec = get_test_tile_spec(alg, 100);
        let _ = group.bench_function(format!("plot_{alg:?}"), |b| {
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
    tile.plot(black_box(512));

    let mut bench = |colourer: Instance| {
        let _ = group.bench_function(format!("colour_{colourer}"), |b| {
            let filename = "/dev/null";
            b.iter_batched_ref(
                || render::factory(render::Selection::Png),
                |r| {
                    let _ = r.render_file(filename, &tile, black_box(colourer));
                },
                BatchSize::SmallInput,
            );
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
    tiles.par_iter_mut().for_each(|t| t.plot(512));

    let _ = group.bench_function("join", |b| {
        b.iter(|| Tile::join(&single, black_box(&tiles)).unwrap());
    });
}

criterion_group!(tiles, tile_join);

// //////////////////////////////////////////////////////////////////////////////////////////

criterion_main!(fractals, colourers, tiles);
