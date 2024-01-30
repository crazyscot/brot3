use std::str::FromStr;

use brot3::colouring::{self, OutputsRgb8, Selection};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use strum::VariantNames;

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
    colouring::Selection::VARIANTS
        .iter()
        .for_each(|i| bench(colouring::Selection::from_str(i).unwrap()));
}

criterion_group!(colourers, colour_pixel);
criterion_main!(colourers);
