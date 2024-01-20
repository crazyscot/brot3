use brot3::fractal::{self, Algorithm, Point, PointData, SelectionFDiscriminants, Tile, TileSpec};
use brot3::render::{self, Renderer, SelectionRDiscriminants};

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
    alg(SelectionFDiscriminants::Original, TEST_POINT_M2);
    alg(SelectionFDiscriminants::Mandel3, TEST_POINT_M3);
}

const TEST_TILE_SPEC: TileSpec = TileSpec {
    origin: Point { re: -1.0, im: 0.0 },
    axes: Point { re: 4.0, im: 4.0 },
    width: 300,
    height: 300,
};

fn tile(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiles");
    group.bench_function("tile_Original", |b| {
        let alg = fractal::factory(SelectionFDiscriminants::Original);
        b.iter_batched_ref(
            || Tile::new(&TEST_TILE_SPEC, &alg, 0),
            |t| {
                t.plot(black_box(512));
            },
            BatchSize::SmallInput,
        );
    });
}

const PALETTE_TILE_SPEC: TileSpec = TileSpec {
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
};

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
    group.bench_function("colour_tile_mandy", |b| {
        let alg = fractal::factory(SelectionFDiscriminants::Original);
        let mut tile = Tile::new(&PALETTE_TILE_SPEC, &alg, 0);
        tile.plot(384);
        let png = render::factory(SelectionRDiscriminants::Png, "/dev/null");
        b.iter(|| png.render(black_box(&tile)));
    });
}

criterion_group!(fractals, iteration, tile);
criterion_group!(palettes, colour_pixel, colour_tile);
criterion_main!(fractals, palettes);
