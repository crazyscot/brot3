use brot3::fractal::{mandelbrot_pixel, mandelbrot_prepare, PlotSpec, Point, PointData, Tile};
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

const TEST_TILE_SPEC: PlotSpec = PlotSpec {
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

criterion_group!(benches, pixel, tile);
criterion_main!(benches);
