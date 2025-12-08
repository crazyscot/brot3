#![allow(missing_docs)]

fn main() {
    // Run registered benchmarks.
    divan::main();
}

use std::sync::LazyLock;

use divan::black_box;
use shader::{
    Vec2,
    colourspace::RgbVec,
    exponentiation::{
        ComplexPower, Exponentiator, IntegerPower, Power2, Power3, Power4, RealPower,
    },
    vec2,
};
use shader_common::{
    Complex, Flags, FragmentConstants, Palette, PushExponent, Size,
    data::PointResult,
    enums::{Algorithm, Colourer},
};
use strum::VariantArray as _;

#[divan::bench]
fn ___warm_up() {
    // this is a hack to ensure the binary and libraries are fully loaded.
    // without it, the first run of the first benchmark - and certain others - are outliers.
    let _ = IntegerPower(2).apply_to(Complex::new(0., 0.));
    let _ = Power2 {}.apply_to(Complex::new(0., 0.));
}

#[divan::bench(args = Algorithm::VARIANTS)]
fn fractal(alg: Algorithm) -> PointResult {
    let consts = FragmentConstants {
        flags: Flags::NEEDS_REITERATE,
        viewport_translate: vec2(0., 0.),
        viewport_zoom: 0.3,
        size: Size::new(1024, 1024),
        buffer_size: Size::new(1024, 1024),
        max_iter: 10,
        algorithm: alg,
        exponent: PushExponent::from(2),
        palette: Palette::default(),
        inspector_point_pixel_address: Vec2::default(),
    };
    shader::fractal::render(&consts, black_box(vec2(0.5, 0.5)))
}

#[divan::bench(args = Colourer::VARIANTS)]
fn colour(col: Colourer) -> RgbVec {
    let consts = FragmentConstants {
        flags: Flags::NEEDS_REITERATE,
        viewport_translate: vec2(0., 0.),
        viewport_zoom: 0.3,
        size: Size::new(1024, 1024),
        buffer_size: Size::new(1024, 1024),
        max_iter: 10,
        algorithm: Algorithm::default(),
        exponent: PushExponent::from(2),
        palette: Palette::default().with_colourer(col),
        inspector_point_pixel_address: Vec2::default(),
    };
    let data = PointResult::new_outside(3, 5.423, 0.123, 1., 2.);
    shader::colour::colour_data(black_box(data), &consts, 0.0)
}

#[derive(Copy, Clone, derive_more::Debug)]
enum Ewrap {
    #[debug("SP2")]
    P2(Power2),
    #[debug("SP3")]
    P3(Power3),
    #[debug("SP4")]
    P4(Power4),
    #[debug("I{}", _0.0)]
    Int(IntegerPower),
    #[debug("R{}", _0.0)]
    Real(RealPower),
    #[debug("C{}", _0.0)]
    Complex(ComplexPower),
}
impl Exponentiator for Ewrap {
    fn apply_to(self, z: Complex) -> Complex {
        match self {
            Ewrap::P2(p) => p.apply_to(z),
            Ewrap::P3(p) => p.apply_to(z),
            Ewrap::P4(p) => p.apply_to(z),
            Ewrap::Int(p) => p.apply_to(z),
            Ewrap::Real(p) => p.apply_to(z),
            Ewrap::Complex(p) => p.apply_to(z),
        }
    }

    fn power(self) -> f32 {
        todo!()
    }

    fn log2(self) -> f32 {
        todo!()
    }
}

static EXP_CASES: LazyLock<Vec<Ewrap>> = LazyLock::new(|| {
    vec![
        Ewrap::P2(Power2 {}),
        Ewrap::P3(Power3 {}),
        Ewrap::P4(Power4 {}),
        Ewrap::Int(IntegerPower(2)),
        Ewrap::Int(IntegerPower(3)),
        Ewrap::Int(IntegerPower(4)),
        Ewrap::Real(RealPower(2.0)),
        Ewrap::Real(RealPower(2.1)),
        Ewrap::Real(RealPower(3.0)),
        Ewrap::Complex(ComplexPower(Complex::from(2.0))),
        Ewrap::Complex(ComplexPower(Complex::from(2.1))),
        Ewrap::Complex(ComplexPower(Complex { re: 2.1, im: 0.1 })),
    ]
});

const EXP_INPUT: Complex = Complex { re: 0.1, im: 0.5 };

#[divan::bench(args = LazyLock::force(&EXP_CASES) )]
fn exponentiation(e: Ewrap) {
    let _ = e.apply_to(black_box(EXP_INPUT));
}
