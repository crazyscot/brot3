use brot3::colouring::{self, ColourerInstance, OutputsRgb8, Selection};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use strum::IntoEnumIterator;

fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("colourers");
    let mut bench = |sel: Selection| {
        let instance = colouring::factory(sel);
        group.bench_function(format!("{sel:?}"), |b| {
            b.iter(|| {
                instance.colour_rgb8(black_box(42.0));
            });
        });
    };
    ColourerInstance::iter().for_each(|i| bench(i.into()));
}

criterion_group!(colourers, colour_pixel);
criterion_main!(colourers);
