use brot3::fractal::{self, Point, Tile, TileSpec};
use brot3::render::{self, Renderer};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

criterion_group!(palettes, colour_pixel, colour_tile);
criterion_main!(palettes);
