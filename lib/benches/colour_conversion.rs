#![allow(missing_docs)]

fn main() {
    // Run registered benchmarks.
    divan::main();
}

use brot3_lib::util::{Hsl, RgbVec};
use divan::black_box;

#[divan::bench]
fn ___warm_up() -> RgbVec {
    // this is a hack to try ensure everything is preloaded and avoid outliers
    let hsl = Hsl::new(42.0, 67.0, 123.0);
    RgbVec::from(black_box(hsl))
}

#[divan::bench]
fn hsl_to_rgb() -> RgbVec {
    let hsl = Hsl::new(128.0, 100.0, 100.0);
    RgbVec::from(black_box(hsl))
}
