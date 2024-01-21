use brot3::fractal::{self, Algorithm, Point, PointData, Tile, TileSpec};
use brot3::render::{self, Renderer};

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

fn test_tile_spec() -> TileSpec {
    TileSpec {
        origin: Point { re: -1.0, im: 0.0 },
        axes: Point { re: 4.0, im: 4.0 },
        width: 300,
        height: 300,
        algorithm: fractal::factory(fractal::Selection::Original),
    }
}

fn tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    let spec = test_tile_spec();
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

fn palette_tile_spec() -> TileSpec {
    TileSpec {
        origin: Point {
            re: -0.0946,
            im: 1.0105,
        },
        axes: Point {
            re: 0.282,
            im: 0.282,
        },
        width: 300,
        height: 300,
        algorithm: fractal::factory(fractal::Selection::Original),
    }
}

// TODO: We should be able to macrify this for additional palettes.
// Could we in fact autobench all the palettes?
fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("palettes");

    group.bench_function("colour_pixel_mandy", |b| {
        b.iter(|| render::colour_temp(black_box(42.0)));
    });
}

fn colour_tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("palettes");
    let spec = palette_tile_spec();
    group.bench_function("colour_tile_mandy", |b| {
        let mut tile = Tile::new(&spec, 0);
        tile.plot(384);
        let png = render::factory(render::Selection::Png, "/dev/null");
        b.iter(|| png.render(black_box(&tile)));
    });
}

criterion_group!(fractals, iteration, tile);
criterion_group!(palettes, colour_pixel, colour_tile);
criterion_main!(fractals, palettes);
