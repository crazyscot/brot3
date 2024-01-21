use brot3::fractal::{self, Algorithm, Point, PointData, Tile, TileSpec};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

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
}

fn get_test_tile_spec() -> TileSpec {
    TileSpec::new(
        Point { re: -1.0, im: 0.0 },
        Point { re: 4.0, im: 4.0 },
        (1000, 650),
        fractal::factory(fractal::Selection::Original),
    )
}

fn tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let spec = get_test_tile_spec();
    group.bench_function("tile_Original", |b| {
        b.iter_batched_ref(
            || Tile::new(&spec, 0),
            |t| {
                t.plot(black_box(512));
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(fractals, iteration, tile);
criterion_main!(fractals);
