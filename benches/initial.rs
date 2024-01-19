use brot3::fractal::{mandelbrot_pixel, mandelbrot_prepare, Point, PointData, Tile, TileSpec};
use brot3::render;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT: Point = Point::new(-0.158_653_6, 1.034_804);

fn pixel(c: &mut Criterion) {
    let template = PointData::new(TEST_POINT);
    c.bench_function("mandelbrot_pixel", |b| {
        let mut point = template;
        mandelbrot_prepare(&mut point);
        b.iter(|| mandelbrot_pixel(&mut point, black_box(512)))
    });
}

const TEST_TILE_SPEC: TileSpec = TileSpec {
    origin: Point { re: -1.0, im: 0.0 },
    axes: Point { re: 4.0, im: 4.0 },
    width: 300,
    height: 300,
};

fn tile(c: &mut Criterion) {
    c.bench_function("mandelbrot_tile", |b| {
        let mut tile = Tile::new(&TEST_TILE_SPEC, 0);
        tile.prepare();
        b.iter(|| {
            tile.plot(black_box(512));
        });
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

fn colour_pixel(c: &mut Criterion) {
    c.bench_function("colour_pixel_mandy", |b| {
        b.iter(|| render::colour_temp(black_box(42.0)));
    });
}

fn colour_tile(c: &mut Criterion) {
    c.bench_function("colour_tile_mandy", |b| {
        let mut tile = Tile::new(&PALETTE_TILE_SPEC, 0);
        tile.prepare();
        tile.plot(384);
        let png = render::factory(render::WhichRenderer::Png, "/dev/null");
        b.iter(|| png.render(black_box(&tile)));
    });
}

criterion_group!(benches, pixel, tile, colour_pixel, colour_tile);
criterion_main!(benches);
