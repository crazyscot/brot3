fn main() {
    // Run registered benchmarks.
    divan::main();
}

use divan::black_box;
use shader::exponentiation::{Exp2, ExpFloat, ExpIntN, Exponentiator as _};
use shader_common::{
    Algorithm, Colourer, FragmentConstants, Palette, PointResult, PushExponent, complex::Complex,
};
use shader_util::{Size, Vec3, vec2};
use strum::VariantArray as _;

#[divan::bench]
fn ___warm_up() {
    // this is a hack to ensure the binary and libraries are fully loaded.
    // without it, the first run of the first benchmark - and certain others - are outliers.
    let _ = ExpIntN(2).apply_to(Complex::new(0., 0.));
}

#[divan::bench(args = Algorithm::VARIANTS)]
fn fractal(alg: Algorithm) -> PointResult {
    let consts = FragmentConstants {
        viewport_translate: vec2(0., 0.),
        viewport_zoom: 0.3,
        size: Size::new(1024, 1024),
        max_iter: 10,
        needs_reiterate: true.into(),
        algorithm: alg,
        exponent: PushExponent::from(2),
        palette: Palette::default(),
    };
    shader::fractal::render(&consts, black_box(vec2(0.5, 0.5)))
}

#[divan::bench(args = Colourer::VARIANTS)]
fn colour(col: Colourer) -> Vec3 {
    let consts = FragmentConstants {
        viewport_translate: vec2(0., 0.),
        viewport_zoom: 0.3,
        size: Size::new(1024, 1024),
        max_iter: 10,
        needs_reiterate: true.into(),
        algorithm: Algorithm::default(),
        exponent: PushExponent::from(2),
        palette: Palette {
            colourer: col,
            ..Default::default()
        },
    };
    let data = PointResult::new(false, 3, 5.423);
    shader::colour::colour_data(black_box(data), &consts)
}

#[divan::bench(args = [0, 1, 2])]
fn exponentiation(which: u32) -> Complex {
    let z = Complex::new(0.1, 0.5);
    match which {
        0 => Exp2 {}.apply_to(black_box(z)),
        1 => ExpIntN(2).apply_to(black_box(z)),
        2 => ExpFloat(2.).apply_to(black_box(z)),
        _ => unreachable!(),
    }
}
