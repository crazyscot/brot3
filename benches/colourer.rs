use brot3::colouring::{self, Instance, OutputsRgb8, Selection::*};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn colour_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("colourers");
    let mut bench = |instance: Instance| {
        group.bench_function(format!("{}", instance), |b| {
            b.iter(|| {
                instance.colour_rgb8(black_box(42.0), 256);
            });
        });
    };
    // We run only a selection of algorithms through the full benchmarker
    // (See also IAI, which runs them all.)
    let selection = [LinearRainbow, LchGradient, Mandy, WhiteFade];
    selection
        .iter()
        .for_each(|i| bench(colouring::Instance::from_repr(*i as usize).unwrap()));
}

criterion_group!(colourers, colour_pixel);
criterion_main!(colourers);
