#![allow(missing_docs)]

fn main() {
    // Run registered benchmarks.
    divan::main();
}

use divan::black_box;
use shader_util::colourspace::{Hsl, Lab, Lch, Rgb, Vec3Rgb};

#[divan::bench]
fn ___warm_up() -> Lab {
    // this is a hack to try ensure everything is preloaded and avoid outliers
    let lch = Lch::new(42.0, 67.0, 123.0);
    Lab::from(black_box(lch))
}

#[divan::bench]
fn hsl_to_rgb() -> Rgb {
    let hsl = Hsl::new(128.0, 100.0, 100.0);
    Rgb::from(black_box(hsl))
}

#[divan::bench]
fn lch_to_rgb() -> Vec3Rgb {
    let lch = Lch::new(42.0, 67.0, 123.0);
    Vec3Rgb::from(black_box(lch))
}

#[divan::bench]
fn lch_to_lab() -> Lab {
    let lch = Lch::new(42.0, 67.0, 123.0);
    Lab::from(black_box(lch))
}

#[divan::bench]
fn lab_to_rgb() -> Vec3Rgb {
    let lab = Lab::new(57.0, -42.0, 87.0);
    Vec3Rgb::from(black_box(lab))
}
