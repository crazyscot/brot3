use brot3::fractal::{mandelbrot_pixel, mandelbrot_prepare, Point, PointData};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// A point (found by experiment) that's in the set but not in the special-case cut-off regions
const TEST_POINT: Point = Point::new(-0.158_653_6, 1.034_804);

fn pixel(c: &mut Criterion) {
    let template = PointData::new(TEST_POINT);
    c.bench_function("mandelbrot_pixel", |b| {
        let mut point = template;
        mandelbrot_prepare(&mut point);
        b.iter(|| mandelbrot_pixel(&mut point, black_box(16384)))
    });
}

// also tile, colour

criterion_group!(benches, pixel);
criterion_main!(benches);
